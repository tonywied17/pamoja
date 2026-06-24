//! A worked gateway: drive the dashboard from a real pamoja profile, with no mock.
//!
//! This is the shape to copy into a real project. It assembles a profile's controller,
//! samples a sensor on a loop, reports each reading into a [`Fleet`], applies the control
//! commands the dashboard queues, and serves the dashboard from that fleet. Swap the
//! stand-in sensor for a real `pamoja-sensors` driver, and (to also publish telemetry
//! upstream) tick the async `pamoja_profile::Node` instead of its controller directly.
//!
//! Run: `cargo run -p pamoja-dashboard --example gateway`

use std::thread;
use std::time::Duration;

use pamoja_dashboard::{
    Assets, Auth, Command, Fleet, LinkKind, Reading, Sensor, Server, Status, Trend,
};
use pamoja_profile::{Alert, Profile};

fn main() -> std::process::ExitCode {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8788".to_owned());

    // The fleet the dashboard renders and the sampling loop fills: a farm node with a soil
    // sensor (read automatically) and a drip valve (controlled from the dashboard).
    let fleet = Fleet::builder()
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
        .build();

    // The sampling loop. A real project ticks its profile here on the power schedule; this
    // drifts a stand-in soil reading so the page is alive, judges it with the profile's
    // controller, and applies any control command the dashboard queued.
    let worker = fleet.clone();
    thread::spawn(move || {
        let profile = Profile::irrigation_node();
        let mut control = profile.controller();
        let mut tick = 0.0f32;
        loop {
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

            // Apply control commands the dashboard queued. The valve is dashboard-driven, so
            // a real gateway would move the hardware here; this reflects the new state back.
            for command in worker.take_commands() {
                if let Command::Actuate { target, action } = &command {
                    if target == "field/valve" {
                        let on = action == "open";
                        worker.report_reading(
                            "field",
                            "valve",
                            Reading::new("drip_valve", if on { 1.0 } else { 0.0 }, "state")
                                .with_state(format!("state.{action}"))
                                .with_actions(["open", "closed"]),
                        );
                    }
                }
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
