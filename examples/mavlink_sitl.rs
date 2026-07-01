//! Talk to a MAVLink autopilot with no hardware.
//!
//! A ground station and a software-in-the-loop autopilot are joined by an in-process link,
//! and both sign their traffic. The ground station waits for the vehicle's heartbeat, arms
//! it with a signed command, and reads back the signed acknowledgement, exercising the
//! whole connect, command, and telemetry path the way a real link would.

use pamoja_mavlink::dialect::{self, CommandAck, CommandLong, Heartbeat, Message};
use pamoja_mavlink::link::{Connection, MemoryLink, SitlAutopilot};
use pamoja_mavlink::signing::{Signer, Verifier};

// A signing key shared by the ground station and the vehicle, pinned at provisioning time.
const KEY: [u8; 32] = [0x5A; 32];

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), pamoja_mavlink::MavlinkError> {
    let (gcs_link, vehicle_link) = MemoryLink::pair();

    // The vehicle: system 1, autopilot component 1, signing on.
    let mut vehicle = SitlAutopilot::new(vehicle_link, 1, 1)
        .secured(Signer::new(KEY, 1, 10_000), Verifier::new(KEY));

    // The ground station: the conventional GCS system 255, component 190, signing on.
    let mut gcs = Connection::new(gcs_link, 255, 190)
        .with_signer(Signer::new(KEY, 2, 20_000))
        .with_verifier(Verifier::new(KEY));

    // The vehicle announces itself; the ground station waits for that heartbeat.
    vehicle.emit_heartbeat().await?;
    let frame = gcs.recv().await?;
    let heartbeat = Heartbeat::decode(frame.payload())?;
    println!(
        "heartbeat from system {}: type {}, autopilot {}, status {}",
        frame.system_id(),
        heartbeat.type_,
        heartbeat.autopilot,
        heartbeat.system_status,
    );

    // Arm the vehicle with a signed command.
    let arm = CommandLong {
        param1: 1.0,
        param2: 0.0,
        param3: 0.0,
        param4: 0.0,
        param5: 0.0,
        param6: 0.0,
        param7: 0.0,
        command: dialect::mav_cmd::COMPONENT_ARM_DISARM,
        target_system: 1,
        target_component: 1,
        confirmation: 0,
    };
    gcs.send(&arm).await?;
    println!("sent signed arm command");

    // The vehicle verifies the command and acknowledges it.
    vehicle.serve_once().await?;

    // The ground station reads and verifies the acknowledgement.
    let frame = gcs.recv().await?;
    let ack = CommandAck::decode(frame.payload())?;
    println!(
        "acknowledgement for command {}: result {} (signed: {})",
        ack.command,
        ack.result,
        frame.is_signed(),
    );

    assert_eq!(frame.message_id(), CommandAck::ID);
    assert_eq!(ack.result, dialect::mav_result::ACCEPTED);
    println!("vehicle armed over a signed MAVLink link, no hardware");
    Ok(())
}
