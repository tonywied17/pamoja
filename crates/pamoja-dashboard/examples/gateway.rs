//! A worked gateway: drive the dashboard from a real pamoja profile, with no mock.
//!
//! This is the shape to copy into a real project. It restores a persisted fleet on boot,
//! assembles a profile's controller, samples a sensor on a loop, surfaces a node when it is
//! discovered, applies the control commands the dashboard queues (honouring any hardware
//! binding), persists changes, and serves the dashboard from that fleet. Swap the stand-in
//! sensor for a real `pamoja-sensors` driver, and (to also publish telemetry upstream) tick
//! the async `pamoja_profile::Node` instead of its controller directly.
//!
//! Run: `cargo run -p pamoja-dashboard --example gateway`

use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use pamoja_dashboard::{
    Assets, Auth, Command, Fleet, LinkKind, Reading, Sensor, Server, State, StateSource, Status,
    Trend,
};
use pamoja_profile::{Alert, Profile};

// Where this example persists its fleet, so provisioning and the last valve state survive a
// restart. A real gateway uses its own durable storage.
fn state_path() -> PathBuf {
    std::env::temp_dir().join("pamoja-gateway-state.json")
}

// Restores a saved fleet if one exists, otherwise builds the initial structure.
fn build_fleet() -> Fleet {
    if let Ok(text) = std::fs::read_to_string(state_path()) {
        if let Ok(state) = State::from_json(&text) {
            println!("gateway: restored fleet from {}", state_path().display());
            return Fleet::from_state(state);
        }
    }
    Fleet::builder()
        .org("farm", "Pamoja farm")
        .group("farm", "field", "Field node", LinkKind::Lora)
        .sensor(
            "field",
            Sensor::new(
                "soil",
                Reading::new("soil_moisture", 60.0, "percent").with_band(40.0, 80.0),
            ),
        )
        .sensor(
            "field",
            Sensor::new(
                "valve",
                Reading::new("drip_valve", 0.0, "state")
                    .with_state("state.closed")
                    .with_actions(["open", "closed"]),
            ),
        )
        .build()
}

fn save(fleet: &mut Fleet) {
    if let Ok(json) = fleet.snapshot().to_json() {
        let _ = std::fs::write(state_path(), json);
    }
}

fn main() -> std::process::ExitCode {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8788".to_owned());

    let fleet = build_fleet();

    // The sampling loop. A real project ticks its profile here on the power schedule; this
    // drifts a stand-in soil reading, judges it with the profile's controller, surfaces a
    // discovered node, and applies any control command the dashboard queued.
    let mut worker = fleet.clone();
    thread::spawn(move || {
        let profile = Profile::irrigation_node();
        let mut control = profile.controller();
        let mut tick = 0.0f32;
        let mut step = 0u32;
        loop {
            step += 1;
            tick += 0.4;
            let moisture = 60.0 + 25.0 * tick.sin();
            let status = match control.evaluate(moisture).alert {
                Some(Alert::OutOfRange { .. }) => Status::Alarm,
                Some(_) => Status::Warn,
                None => Status::Ok,
            };
            worker.report_reading(
                "field",
                "soil",
                Reading::new("soil_moisture", moisture, "percent")
                    .with_band(40.0, 80.0)
                    .with_status(status)
                    .with_trend(Trend::Steady),
            );

            // Discovery: a humidity node joins after a few cycles; it appears in the dashboard.
            if step == 8 {
                println!("gateway: a humidity node joined; surfacing it");
                worker.add_sensor(
                    "field",
                    Sensor::new(
                        "humidity",
                        Reading::new("humidity", 55.0, "percent").with_band(30.0, 70.0),
                    ),
                );
                save(&mut worker);
            }

            // Apply control commands the dashboard queued, then persist the change.
            let commands = worker.take_commands();
            if !commands.is_empty() {
                for command in &commands {
                    match command {
                        Command::Actuate { target, action } if target == "field/valve" => {
                            let on = action == "open";
                            worker.report_reading(
                                "field",
                                "valve",
                                Reading::new("drip_valve", if on { 1.0 } else { 0.0 }, "state")
                                    .with_state(format!("state.{action}"))
                                    .with_actions(["open", "closed"]),
                            );
                        }
                        Command::AddSensor {
                            binding: Some(binding),
                            sensor,
                            ..
                        } => println!("gateway: bind sensor {} via {binding}", sensor.id),
                        other => println!("gateway: applying {other:?}"),
                    }
                }
                save(&mut worker);
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let secret = Auth::generate_secret();
    println!("gateway: pairing code (unlock control with this): {secret}");
    println!("gateway: serving on http://{addr}");
    match Server::new(fleet, Assets::Embedded)
        .with_pairing_secret(secret)
        .run(&addr)
    {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("gateway: could not serve on {addr}: {err}");
            std::process::ExitCode::FAILURE
        }
    }
}
