//! A real fleet source: a project pushes readings in, the dashboard reads them out, and
//! control commands queue back for the project to apply.
//!
//! The dashboard renders whatever implements [`StateSource`]. [`Mock`](crate::Mock) is the
//! hardware-free demo; `Fleet` is the real one. Sensing in this SDK is async and the server
//! is synchronous, so the project owns its own sampling loop - ticking its profiles and
//! nodes on their power schedule - and pushes each result into a `Fleet` with the report
//! methods. The dashboard reads the latest with [`snapshot`](StateSource::snapshot), and
//! authenticated control [`Command`]s queue for the project to drain with
//! [`take_commands`](Fleet::take_commands) and apply to real hardware, then reflect the
//! result back through the report methods. The device stays authoritative; provisioning and
//! actuation are also applied optimistically so the UI updates at once.
//!
//! A `Fleet` is cheap to [`Clone`] (it shares one inner state), so one handle drives the
//! [`Server`](crate::Server) while another stays with the sampling loop.
//!
//! # Examples
//!
//! ```
//! use pamoja_dashboard::{Fleet, LinkKind, Reading, Sensor, StateSource, Status};
//!
//! let fleet = Fleet::builder()
//!     .org("clinic", "Kano clinic")
//!     .group("clinic", "fridges", "Cold chain", LinkKind::Cellular)
//!     .sensor(
//!         "fridges",
//!         Sensor::new("fridge-1", Reading::new("fridge_temp", 4.5, "celsius").with_band(2.0, 8.0)),
//!     )
//!     .build();
//!
//! // The sampling loop pushes a fresh reading; the dashboard sees it.
//! fleet.report_reading(
//!     "fridges",
//!     "fridge-1",
//!     Reading::new("fridge_temp", 9.2, "celsius").with_band(2.0, 8.0).with_status(Status::Alarm),
//! );
//! let mut handle = fleet.clone();
//! assert_eq!(handle.snapshot().status, Status::Alarm);
//! ```

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::command::{Command, CommandError};
use crate::source::StateSource;
use crate::state::{EventRecord, Group, Link, LinkKind, Mode, Org, Reading, Sensor, State, Status};

// How many recent values each sensor keeps for its sparkline.
const HISTORY: usize = 32;

struct Inner {
    state: State,
    commands: Vec<Command>,
    started: Instant,
}

/// A real fleet a project fills and the dashboard renders. Clone to share one between the
/// serving layer and the sampling loop.
#[derive(Clone)]
pub struct Fleet {
    inner: Arc<Mutex<Inner>>,
}

impl Fleet {
    /// Starts building a fleet's initial structure.
    ///
    /// # Returns
    ///
    /// An empty [`FleetBuilder`].
    pub fn builder() -> FleetBuilder {
        FleetBuilder { orgs: Vec::new() }
    }

    fn with_state(state: State) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                state,
                commands: Vec::new(),
                started: Instant::now(),
            })),
        }
    }

    /// Pushes a fresh reading for a sensor, appending it to the sensor's history.
    ///
    /// # Arguments
    ///
    /// * `group` - the sensor's group id.
    /// * `sensor` - the sensor id.
    /// * `reading` - the new reading (the caller sets its status and band).
    pub fn report_reading(&self, group: &str, sensor: &str, reading: Reading) {
        let mut inner = self.inner.lock().expect("fleet lock");
        if let Some(target) = sensor_mut(&mut inner.state, group, sensor) {
            let value = reading.value;
            target.reading = reading;
            target.history.push(value);
            let len = target.history.len();
            if len > HISTORY {
                target.history.drain(0..len - HISTORY);
            }
        }
        recompute(&mut inner.state);
    }

    /// Records a recent event for a sensor, newest first.
    ///
    /// # Arguments
    ///
    /// * `group` - the sensor's group id.
    /// * `sensor` - the sensor id.
    /// * `event` - the event to record.
    pub fn report_event(&self, group: &str, sensor: &str, event: EventRecord) {
        let mut inner = self.inner.lock().expect("fleet lock");
        if let Some(target) = sensor_mut(&mut inner.state, group, sensor) {
            target.events.insert(0, event);
            target.events.truncate(8);
        }
        recompute(&mut inner.state);
    }

    /// Updates a group's link status (kind, signal strength, online).
    ///
    /// # Arguments
    ///
    /// * `group` - the group id.
    /// * `link` - the new link status.
    pub fn report_link(&self, group: &str, link: Link) {
        let mut inner = self.inner.lock().expect("fleet lock");
        if let Some(target) = group_mut(&mut inner.state, group) {
            target.link = link;
        }
        recompute(&mut inner.state);
    }

    /// Updates a sensor's power mode and battery state of charge.
    ///
    /// # Arguments
    ///
    /// * `group` - the sensor's group id.
    /// * `sensor` - the sensor id.
    /// * `mode` - the work cadence the sensor's node is running at.
    /// * `battery` - the state of charge in `[0.0, 1.0]`, or `None` if it has no battery.
    pub fn report_power(&self, group: &str, sensor: &str, mode: Mode, battery: Option<f32>) {
        let mut inner = self.inner.lock().expect("fleet lock");
        if let Some(target) = sensor_mut(&mut inner.state, group, sensor) {
            target.mode = mode;
            target.battery = battery;
        }
    }

    /// Drains the control commands queued since the last call, for the project to apply to
    /// real hardware and persist, then reflect back through the report methods.
    ///
    /// # Returns
    ///
    /// The commands accepted since the last drain, in order.
    pub fn take_commands(&self) -> Vec<Command> {
        let mut inner = self.inner.lock().expect("fleet lock");
        std::mem::take(&mut inner.commands)
    }
}

impl StateSource for Fleet {
    fn snapshot(&mut self) -> State {
        let mut inner = self.inner.lock().expect("fleet lock");
        let uptime = inner.started.elapsed().as_secs();
        inner.state.uptime_secs = Some(uptime);
        inner.state.clone()
    }

    fn command(&mut self, command: &Command) -> Result<(), CommandError> {
        let mut inner = self.inner.lock().expect("fleet lock");
        let outcome = apply(&mut inner.state, command);
        if outcome.is_ok() {
            inner.commands.push(command.clone());
            recompute(&mut inner.state);
        }
        outcome
    }
}

// Applies a command optimistically to the held state, mirroring the device's eventual
// effect so the UI updates at once; the queued copy lets the project make it real.
fn apply(state: &mut State, command: &Command) -> Result<(), CommandError> {
    match command {
        Command::Actuate { target, action } => {
            let (group, sensor) = target.split_once('/').ok_or(CommandError::UnknownTarget)?;
            let reading = sensor_mut(state, group, sensor)
                .map(|s| &mut s.reading)
                .ok_or(CommandError::UnknownTarget)?;
            match &reading.actions {
                Some(actions) if actions.iter().any(|a| a == action) => {
                    reading.state = Some(format!("state.{action}"));
                    Ok(())
                }
                Some(_) => Err(CommandError::InvalidAction),
                None => Err(CommandError::Unsupported),
            }
        }
        Command::AddGroup { org, group } => match org_mut(state, org) {
            Some(target) => {
                target.groups.push(group.clone());
                Ok(())
            }
            None => Err(CommandError::UnknownTarget),
        },
        Command::RemoveGroup { id } => {
            for org in &mut state.orgs {
                org.groups.retain(|g| g.id != *id);
            }
            Ok(())
        }
        Command::AddSensor { group, sensor } => match group_mut(state, group) {
            Some(target) => {
                target.sensors.push(sensor.clone());
                Ok(())
            }
            None => Err(CommandError::UnknownTarget),
        },
        Command::RemoveSensor { target } => {
            let (group_id, sensor_id) = target.split_once('/').unwrap_or(("", target));
            for org in &mut state.orgs {
                for group in &mut org.groups {
                    if group.id == group_id {
                        group.sensors.retain(|s| s.id != sensor_id);
                    }
                }
            }
            Ok(())
        }
    }
}

fn recompute(state: &mut State) {
    for org in &mut state.orgs {
        for group in &mut org.groups {
            group.recompute_status();
        }
    }
    state.recompute_status();
}

fn org_mut<'a>(state: &'a mut State, org: &str) -> Option<&'a mut Org> {
    state.orgs.iter_mut().find(|o| o.id == org)
}

fn group_mut<'a>(state: &'a mut State, group: &str) -> Option<&'a mut Group> {
    state
        .orgs
        .iter_mut()
        .flat_map(|o| &mut o.groups)
        .find(|g| g.id == group)
}

fn sensor_mut<'a>(state: &'a mut State, group: &str, sensor: &str) -> Option<&'a mut Sensor> {
    state
        .orgs
        .iter_mut()
        .flat_map(|o| &mut o.groups)
        .filter(|g| g.id == group)
        .flat_map(|g| &mut g.sensors)
        .find(|s| s.id == sensor)
}

/// Builds a fleet's initial structure: organizations, their groups, and each group's
/// sensors. Parents are referenced by id, so add an org before its groups and a group
/// before its sensors.
pub struct FleetBuilder {
    orgs: Vec<Org>,
}

impl FleetBuilder {
    /// Adds an organization.
    ///
    /// # Arguments
    ///
    /// * `id` - the stable organization id.
    /// * `name` - the human-readable name.
    ///
    /// # Returns
    ///
    /// The builder, for chaining.
    pub fn org(mut self, id: impl Into<String>, name: impl Into<String>) -> Self {
        self.orgs.push(Org {
            id: id.into(),
            name: name.into(),
            groups: Vec::new(),
        });
        self
    }

    /// Adds a group to an organization.
    ///
    /// # Arguments
    ///
    /// * `org` - the id of the organization to add the group to.
    /// * `id` - the stable group id.
    /// * `name` - the human-readable name.
    /// * `kind` - the link the group reports over.
    ///
    /// # Returns
    ///
    /// The builder, for chaining.
    pub fn group(
        mut self,
        org: &str,
        id: impl Into<String>,
        name: impl Into<String>,
        kind: LinkKind,
    ) -> Self {
        if let Some(target) = self.orgs.iter_mut().find(|o| o.id == org) {
            target.groups.push(Group {
                id: id.into(),
                name: name.into(),
                link: Link {
                    kind,
                    strength: 4,
                    online: true,
                },
                status: Status::Ok,
                sensors: Vec::new(),
            });
        }
        self
    }

    /// Adds a sensor to a group.
    ///
    /// # Arguments
    ///
    /// * `group` - the id of the group to add the sensor to.
    /// * `sensor` - the sensor (build its reading with [`Reading`] and [`Sensor::new`]).
    ///
    /// # Returns
    ///
    /// The builder, for chaining.
    pub fn sensor(mut self, group: &str, sensor: Sensor) -> Self {
        for org in &mut self.orgs {
            if let Some(target) = org.groups.iter_mut().find(|g| g.id == group) {
                target.sensors.push(sensor);
                break;
            }
        }
        self
    }

    /// Finishes building, returning a [`Fleet`] ready to serve and report into.
    ///
    /// # Returns
    ///
    /// The assembled fleet.
    pub fn build(mut self) -> Fleet {
        let mut state = State {
            orgs: std::mem::take(&mut self.orgs),
            status: Status::Ok,
            uptime_secs: None,
        };
        recompute(&mut state);
        Fleet::with_state(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fleet() -> Fleet {
        Fleet::builder()
            .org("clinic", "Kano clinic")
            .group("clinic", "fridges", "Cold chain", LinkKind::Cellular)
            .sensor(
                "fridges",
                Sensor::new("fridge-1", Reading::new("fridge_temp", 4.5, "celsius")),
            )
            .sensor(
                "fridges",
                Sensor::new(
                    "valve",
                    Reading::new("drip_valve", 0.0, "state")
                        .with_state("state.closed")
                        .with_actions(["open", "closed"]),
                ),
            )
            .build()
    }

    #[test]
    fn a_reported_reading_shows_in_the_snapshot_with_history() {
        let fleet = fleet();
        fleet.report_reading(
            "fridges",
            "fridge-1",
            Reading::new("fridge_temp", 9.0, "celsius").with_status(Status::Alarm),
        );
        let mut handle = fleet.clone();
        let state = handle.snapshot();
        let sensor = &state.orgs[0].groups[0].sensors[0];
        assert_eq!(sensor.reading.value, 9.0);
        assert_eq!(sensor.history, vec![9.0]);
        assert_eq!(
            state.status,
            Status::Alarm,
            "the alarm reading lifts fleet status"
        );
    }

    #[test]
    fn an_actuate_command_updates_state_and_queues_for_the_project() {
        let mut fleet = fleet();
        fleet
            .command(&Command::Actuate {
                target: "fridges/valve".to_owned(),
                action: "open".to_owned(),
            })
            .expect("valve accepts open");
        let queued = fleet.take_commands();
        assert_eq!(
            queued.len(),
            1,
            "the command is queued for the project to apply"
        );
        let valve = sensor_after(&fleet, "fridges", "valve");
        assert_eq!(valve.reading.state.as_deref(), Some("state.open"));
        // Draining empties the queue.
        assert!(fleet.take_commands().is_empty());
    }

    #[test]
    fn an_invalid_actuate_is_refused_and_not_queued() {
        let mut fleet = fleet();
        assert_eq!(
            fleet.command(&Command::Actuate {
                target: "fridges/fridge-1".to_owned(),
                action: "open".to_owned(),
            }),
            Err(CommandError::Unsupported)
        );
        assert!(fleet.take_commands().is_empty());
    }

    #[test]
    fn provisioning_commands_change_the_structure() {
        let mut fleet = fleet();
        fleet
            .command(&Command::AddSensor {
                group: "fridges".to_owned(),
                sensor: Sensor::new("fridge-2", Reading::new("fridge_temp", 5.0, "celsius")),
            })
            .expect("add sensor to a known group");
        assert!(sensor_present(&fleet, "fridges", "fridge-2"));

        fleet
            .command(&Command::RemoveSensor {
                target: "fridges/fridge-2".to_owned(),
            })
            .expect("remove the sensor");
        assert!(!sensor_present(&fleet, "fridges", "fridge-2"));
    }

    fn sensor_after(fleet: &Fleet, group: &str, sensor: &str) -> Sensor {
        let mut handle = fleet.clone();
        let state = handle.snapshot();
        state
            .orgs
            .iter()
            .flat_map(|o| &o.groups)
            .filter(|g| g.id == group)
            .flat_map(|g| &g.sensors)
            .find(|s| s.id == sensor)
            .expect("sensor")
            .clone()
    }

    fn sensor_present(fleet: &Fleet, group: &str, sensor: &str) -> bool {
        let mut handle = fleet.clone();
        handle
            .snapshot()
            .orgs
            .iter()
            .flat_map(|o| &o.groups)
            .filter(|g| g.id == group)
            .flat_map(|g| &g.sensors)
            .any(|s| s.id == sensor)
    }
}
