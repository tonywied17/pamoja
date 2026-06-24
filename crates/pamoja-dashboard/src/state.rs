//! The language-neutral fleet snapshot a gateway serves to its dashboard.
//!
//! A dashboard often watches more than one node: a clinic with several cold-chain
//! fridges, a co-op with many silos, a watershed of river gauges. So the snapshot is a
//! fleet - organizations, each with sensor groups, each group on its own link and
//! holding its own sensors. Everything human-facing travels as stable keys, stable
//! codes, raw values, and canonical units, identical in every locale; the page renders
//! the words and the formatting at the surface.

use serde::{Deserialize, Serialize};

use pamoja_power::PowerMode;
use pamoja_telemetry::{Event, Level};

/// The health of a sensor, group, or the whole fleet, the basis of the glance-first UI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Everything is within its safe band. Ordered least urgent, so the derived
    /// ordering makes [`Status::worst`] a simple `max`.
    #[default]
    Ok,
    /// Something needs attention but is not yet critical.
    Warn,
    /// A safety threshold has been crossed and action is needed now.
    Alarm,
}

impl Status {
    /// Returns the more urgent of two statuses.
    ///
    /// # Arguments
    ///
    /// * `other` - the status to compare against.
    ///
    /// # Returns
    ///
    /// The most urgent of the two.
    pub fn worst(self, other: Status) -> Status {
        self.max(other)
    }
}

/// The direction a reading is moving, drawn as a trend arrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    /// The reading is rising.
    Rising,
    /// The reading is steady.
    Steady,
    /// The reading is falling.
    Falling,
}

/// A single measured value, named by a stable key and a canonical unit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    /// A stable, language-neutral element key, such as `"soil_moisture"`.
    pub key: String,
    /// The raw measured value, in the canonical unit.
    pub value: f32,
    /// The canonical unit name, such as `"percent"`, `"celsius"`, or `"volt"`.
    pub unit: String,
    /// The health of this reading on its own.
    pub status: Status,
    /// The safe band `[low, high]` in the same unit, drawn as the gauge's safe zone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band: Option<[f32; 2]>,
    /// Which way the reading is moving, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<Trend>,
    /// A discrete state code for a non-numeric reading, such as `"state.open"` for a
    /// valve or `"pump.nominal"` for a pump, which the page renders as a labelled chip.
    /// Numeric readings leave this `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// The discrete actions this reading can be commanded to, such as
    /// `["open", "closed"]` for a valve. Present only on a controllable actuator; a
    /// read-only sensor leaves this `None`, and the page shows control only when it is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<String>>,
}

impl Reading {
    /// Creates a reading in good standing with no band or trend.
    ///
    /// # Arguments
    ///
    /// * `key` - the stable element key.
    /// * `value` - the raw measured value.
    /// * `unit` - the canonical unit name.
    ///
    /// # Returns
    ///
    /// A [`Status::Ok`] reading carrying just the value and unit.
    pub fn new(key: impl Into<String>, value: f32, unit: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value,
            unit: unit.into(),
            status: Status::Ok,
            band: None,
            trend: None,
            state: None,
            actions: None,
        }
    }

    /// Sets the reading's health.
    ///
    /// # Arguments
    ///
    /// * `status` - the health to record.
    ///
    /// # Returns
    ///
    /// The reading, for chaining.
    pub fn with_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    /// Sets the safe band drawn as the gauge's safe zone.
    ///
    /// # Arguments
    ///
    /// * `low` - the bottom of the safe band.
    /// * `high` - the top of the safe band.
    ///
    /// # Returns
    ///
    /// The reading, for chaining.
    pub fn with_band(mut self, low: f32, high: f32) -> Self {
        self.band = Some([low, high]);
        self
    }

    /// Sets the reading's trend arrow.
    ///
    /// # Arguments
    ///
    /// * `trend` - which way the reading is moving.
    ///
    /// # Returns
    ///
    /// The reading, for chaining.
    pub fn with_trend(mut self, trend: Trend) -> Self {
        self.trend = Some(trend);
        self
    }

    /// Sets a discrete state code for a non-numeric reading, rendered as a chip.
    ///
    /// # Arguments
    ///
    /// * `state` - the stable state code, such as `"state.open"`.
    ///
    /// # Returns
    ///
    /// The reading, for chaining.
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Marks the reading as a controllable actuator with the given discrete actions.
    ///
    /// # Arguments
    ///
    /// * `actions` - the action codes a client may command, such as `["open", "closed"]`.
    ///
    /// # Returns
    ///
    /// The reading, for chaining.
    pub fn with_actions(mut self, actions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.actions = Some(actions.into_iter().map(Into::into).collect());
        self
    }
}

/// The severity of a telemetry event, mirrored onto the wire as a stable string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventLevel {
    /// Fine-grained detail.
    Trace,
    /// Diagnostic detail.
    Debug,
    /// A normal, noteworthy event.
    Info,
    /// Something unexpected the node recovered from.
    Warn,
    /// A failure that needs attention.
    Error,
}

impl From<Level> for EventLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::Trace => EventLevel::Trace,
            Level::Debug => EventLevel::Debug,
            Level::Info => EventLevel::Info,
            Level::Warn => EventLevel::Warn,
            Level::Error => EventLevel::Error,
        }
    }
}

/// One recent telemetry event, carried as a stable code the page localizes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRecord {
    /// The event's severity.
    pub level: EventLevel,
    /// The stable, short event code, such as `"battery.low"` or `"link.lost"`.
    pub code: String,
    /// An optional measurement that came with the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f32>,
    /// How many seconds ago the event happened, for a relative "x ago" display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_secs: Option<u64>,
}

impl EventRecord {
    /// Builds a record from a telemetry [`Event`] and how long ago it happened.
    ///
    /// # Arguments
    ///
    /// * `event` - the telemetry event to mirror onto the wire.
    /// * `age_secs` - how many seconds ago it happened, or `None` if unknown.
    ///
    /// # Returns
    ///
    /// The serializable event record.
    pub fn from_event(event: &Event, age_secs: Option<u64>) -> Self {
        Self {
            level: event.level.into(),
            code: event.code.to_owned(),
            value: event.value,
            age_secs,
        }
    }
}

/// The work cadence a node is running at, mirrored from [`PowerMode`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Healthy charge: the normal cadence.
    Active,
    /// Low charge: a stretched cadence to conserve.
    Saver,
    /// Critically low charge: the bare minimum to survive.
    Critical,
}

impl From<PowerMode> for Mode {
    fn from(mode: PowerMode) -> Self {
        match mode {
            PowerMode::Active => Mode::Active,
            PowerMode::Saver => Mode::Saver,
            PowerMode::Critical => Mode::Critical,
        }
    }
}

/// The kind of link a group reports over, shown as a labelled service before the bars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    /// Long-range, low-power radio.
    Lora,
    /// Local WiFi.
    Wifi,
    /// A cellular modem (LTE-M, 2G/4G, or similar).
    Cellular,
    /// A narrowband-IoT cellular link, common for low-power field clinics.
    NbIot,
    /// A satellite uplink.
    Satellite,
    /// Wired Ethernet.
    Ethernet,
    /// A multi-hop radio mesh.
    Mesh,
}

/// A group's connectivity: what it talks over, how strong it is, and whether it is up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// The kind of link.
    pub kind: LinkKind,
    /// Signal strength as a bar count in `0..=4`.
    pub strength: u8,
    /// Whether the group currently has any uplink at all.
    pub online: bool,
}

/// A single sensor: its current reading, recent history, power, and recent events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sensor {
    /// A stable, human-readable sensor identifier, such as `"fridge-1"`.
    pub id: String,
    /// The sensor's current reading.
    pub reading: Reading,
    /// The sensor's battery state of charge in `[0.0, 1.0]`, if it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery: Option<f32>,
    /// The work cadence the sensor's node is running at.
    pub mode: Mode,
    /// Recent values of the reading, oldest first, for a sparkline and min/max.
    pub history: Vec<f32>,
    /// The most recent telemetry events for this sensor, newest first.
    pub events: Vec<EventRecord>,
}

/// A group of sensors sharing one node and one link, such as a clinic's fridges.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    /// A stable group identifier.
    pub id: String,
    /// A human-readable group name, such as `"Kano cold chain"`.
    pub name: String,
    /// The group's link.
    pub link: Link,
    /// The group's overall health, the worst of its sensors.
    pub status: Status,
    /// The sensors in the group.
    pub sensors: Vec<Sensor>,
}

impl Group {
    /// Recomputes the group's [`status`](Group::status) from its sensors and events.
    ///
    /// # Returns
    ///
    /// The group's overall status, also stored back into [`status`](Group::status).
    pub fn recompute_status(&mut self) -> Status {
        let mut overall = if self.link.online {
            Status::Ok
        } else {
            Status::Warn
        };
        for sensor in &self.sensors {
            overall = overall.worst(sensor.reading.status);
            for event in &sensor.events {
                overall = overall.worst(match event.level {
                    EventLevel::Error => Status::Alarm,
                    EventLevel::Warn => Status::Warn,
                    _ => Status::Ok,
                });
            }
        }
        self.status = overall;
        overall
    }
}

/// An organization, such as a health authority or a farming co-op.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Org {
    /// A stable organization identifier.
    pub id: String,
    /// A human-readable organization name.
    pub name: String,
    /// The sensor groups belonging to the organization.
    pub groups: Vec<Group>,
}

/// The complete language-neutral fleet snapshot served at `GET /state`.
///
/// This is the single source the dashboard renders from. It is byte-identical in
/// every locale; the page supplies all words and formatting.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    /// The organizations in the fleet.
    pub orgs: Vec<Org>,
    /// The fleet's overall health, the worst across every group.
    pub status: Status,
    /// Seconds the gateway has been running, if tracked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_secs: Option<u64>,
}

impl State {
    /// Recomputes every group's status and the fleet's overall status.
    ///
    /// # Returns
    ///
    /// The fleet's overall status, also stored back into [`status`](State::status).
    pub fn recompute_status(&mut self) -> Status {
        let mut overall = Status::Ok;
        for org in &mut self.orgs {
            for group in &mut org.groups {
                overall = overall.worst(group.recompute_status());
            }
        }
        self.status = overall;
        overall
    }

    /// Serializes the snapshot to the compact JSON served at `GET /state`.
    ///
    /// # Returns
    ///
    /// The JSON text of the snapshot.
    ///
    /// # Errors
    ///
    /// Returns a [`serde_json::Error`] if the snapshot cannot be serialized, which in
    /// practice only happens on a non-finite float.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sensor(key: &str, value: f32, status: Status) -> Sensor {
        Sensor {
            id: key.to_owned(),
            reading: Reading::new(key, value, "celsius")
                .with_status(status)
                .with_band(2.0, 8.0),
            battery: Some(0.8),
            mode: Mode::Active,
            history: vec![value],
            events: Vec::new(),
        }
    }

    fn fleet(sensor_status: Status, online: bool) -> State {
        State {
            orgs: vec![Org {
                id: "org-1".to_owned(),
                name: "Org One".to_owned(),
                groups: vec![Group {
                    id: "g1".to_owned(),
                    name: "Group One".to_owned(),
                    link: Link {
                        kind: LinkKind::Lora,
                        strength: 3,
                        online,
                    },
                    status: Status::Ok,
                    sensors: vec![sensor("temperature", 5.0, sensor_status)],
                }],
            }],
            status: Status::Ok,
            uptime_secs: Some(3600),
        }
    }

    #[test]
    fn status_worst_picks_the_most_urgent() {
        assert_eq!(Status::Ok.worst(Status::Warn), Status::Warn);
        assert_eq!(Status::Warn.worst(Status::Alarm), Status::Alarm);
    }

    #[test]
    fn recompute_rolls_sensor_status_up_to_group_and_fleet() {
        let mut state = fleet(Status::Alarm, true);
        assert_eq!(state.recompute_status(), Status::Alarm);
        assert_eq!(state.orgs[0].groups[0].status, Status::Alarm);
        assert_eq!(state.status, Status::Alarm);
    }

    #[test]
    fn an_offline_group_is_at_least_a_warning() {
        let mut state = fleet(Status::Ok, false);
        assert_eq!(state.recompute_status(), Status::Warn);
    }

    #[test]
    fn the_fleet_round_trips_through_json() {
        let mut state = fleet(Status::Warn, true);
        state.recompute_status();
        let json = state.to_json().expect("serialize");
        let restored: State = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(state, restored);
    }

    #[test]
    fn the_wire_uses_stable_lowercase_tags() {
        assert_eq!(serde_json::to_string(&Status::Alarm).unwrap(), "\"alarm\"");
        assert_eq!(serde_json::to_string(&LinkKind::Lora).unwrap(), "\"lora\"");
        assert_eq!(serde_json::to_string(&Mode::Saver).unwrap(), "\"saver\"");
    }
}
