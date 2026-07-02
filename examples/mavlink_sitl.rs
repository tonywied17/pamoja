//! Fly a mission on a MAVLink vehicle with no hardware.
//!
//! A software-in-the-loop autopilot runs on one end of an in-process link and a
//! [`Vehicle`](pamoja_mavlink::Vehicle) drives it from the other, both signing their traffic.
//! The vehicle is a pamoja [`Device`]: connecting waits for its heartbeat, and the mission,
//! command, and offboard surfaces run the real MAVLink protocols over the link. This is the
//! whole ground-station path a real PX4 or ArduPilot link takes, exercised with zero hardware;
//! swapping the in-process link for a [`UdpLink`](pamoja_mavlink::UdpLink) or
//! [`TcpLink`](pamoja_mavlink::TcpLink) points the same code at a real autopilot.

use pamoja_core::{Actuator, Device};
use pamoja_mavlink::dialect::{
    mav_cmd, mav_frame, mav_mission_type, MissionItemInt, SetPositionTargetLocalNed,
};
use pamoja_mavlink::link::{MemoryLink, SitlAutopilot};
use pamoja_mavlink::signing::{Signer, Verifier};
use pamoja_mavlink::{Setpoint, Vehicle};

// A signing key shared by the ground station and the vehicle, pinned at provisioning time.
const KEY: [u8; 32] = [0x5A; 32];

fn waypoint(command: u16, lat: i32, lon: i32, alt: f32) -> MissionItemInt {
    MissionItemInt {
        param1: 0.0,
        param2: 0.0,
        param3: 0.0,
        param4: 0.0,
        x: lat,
        y: lon,
        z: alt,
        seq: 0,
        command,
        target_system: 0,
        target_component: 0,
        frame: mav_frame::GLOBAL_RELATIVE_ALT_INT,
        current: 0,
        autocontinue: 1,
        mission_type: mav_mission_type::MISSION,
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (gcs_link, vehicle_link) = MemoryLink::pair();

    // The vehicle: system 1, autopilot component 1, signing on. It serves the command and
    // mission protocols in the background the way a real autopilot would.
    let mut autopilot = SitlAutopilot::new(vehicle_link, 1, 1)
        .secured(Signer::new(KEY, 1, 10_000), Verifier::new(KEY));
    let autopilot = tokio::spawn(async move {
        let _ = autopilot.emit_heartbeat().await;
        while autopilot.serve_once().await.is_ok() {}
    });

    // The ground station drives the vehicle as a pamoja Device, signing its traffic.
    let mut vehicle = Vehicle::new(gcs_link, 255, 190)
        .with_signer(Signer::new(KEY, 2, 20_000))
        .with_verifier(Verifier::new(KEY));

    // Connecting waits for the vehicle's heartbeat and learns its address.
    vehicle.connect().await?;
    println!("connected to {}", vehicle.id());

    // Upload a three-item plan, then read it back and confirm it round-trips.
    let plan = [
        waypoint(mav_cmd::NAV_TAKEOFF, -353_632_610, 1_491_652_300, 10.0),
        waypoint(mav_cmd::NAV_WAYPOINT, -353_631_000, 1_491_653_000, 20.0),
        waypoint(mav_cmd::NAV_WAYPOINT, -353_630_000, 1_491_654_000, 15.0),
    ];
    vehicle.upload_mission(&plan).await?;
    let downloaded = vehicle.download_mission().await?;
    println!(
        "mission round-trip: uploaded {} items, downloaded {}",
        plan.len(),
        downloaded.len()
    );
    assert_eq!(downloaded.len(), plan.len());

    // Send a command and read its result.
    let result = vehicle.arm(true).await?;
    println!("arm command acknowledged with result {result}");

    // Stream one offboard velocity setpoint through the actuator surface.
    let setpoint = SetPositionTargetLocalNed::velocity(
        0,
        mav_frame::LOCAL_NED,
        vehicle.target_system(),
        vehicle.target_component(),
        0.5,
        0.0,
        -0.2,
    );
    vehicle.apply(Setpoint::Local(setpoint)).await?;
    println!("offboard velocity setpoint sent");

    autopilot.abort();
    println!("flew a signed MAVLink mission over the device model, no hardware");
    Ok(())
}
