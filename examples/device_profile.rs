//! A device profile assembled into a ready-to-run cold-chain node, over loopback.
//!
//! A builder picks the `vaccine-fridge-monitor` profile and hands it a sensor, a
//! cooler relay, an in-process link, and a codec. The assembled node does the rest:
//! each cycle it reads the probe, decides with the profile's controller, switches the
//! cooler, and publishes the reading to a gateway, which decodes it. A door left open
//! pushes the temperature into the danger zone and the node raises an excursion
//! alert; as the simulated battery sags, the node reports a longer wait before the
//! next sample. No hardware, no broker. This composes five crates - `pamoja-profile`,
//! `pamoja-kit` and `pamoja-power` (through the profile), `pamoja-codec`, and
//! `pamoja-loopback` - through the same shapes every binding exposes.
//!
//! Run with: `cargo run -p pamoja-examples --example device_profile`

use pamoja_codec::{CborCodec, Codec};
use pamoja_core::{Actuator, Result, Sensor, Transport};
use pamoja_loopback::{LoopbackBroker, LoopbackTransport};
use pamoja_power::PowerMode;
use pamoja_profile::{Node, Profile};

/// A fridge probe that plays back a scripted afternoon: steady, then a door is left
/// open and the temperature climbs out of the safe range, then it recovers.
struct FridgeProbe {
    readings: std::vec::IntoIter<f32>,
}

impl Sensor for FridgeProbe {
    type Reading = f32;

    async fn read(&mut self) -> Result<f32> {
        self.readings.next().ok_or(pamoja_core::Error::Closed)
    }
}

/// A cooler relay. A real one would switch a compressor; this one is a stand-in.
struct Cooler;

impl Actuator for Cooler {
    type Command = bool;

    async fn apply(&mut self, _on: bool) -> Result<()> {
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Pick a profile. The wiring, tuning, and control loop come with it.
    let profile = Profile::vaccine_fridge_monitor();
    println!(
        "assembled '{}', publishing to {}",
        profile.name, profile.topic
    );

    // A gateway and the node share an in-process broker.
    let broker = LoopbackBroker::new();
    let mut gateway = LoopbackTransport::new(broker.clone());
    let mut link = LoopbackTransport::new(broker);
    gateway.connect().await?;
    link.connect().await?;
    gateway.subscribe("cold-chain/#").await?;

    // (probe reading in C, battery state of charge) across the afternoon. A door is
    // left open in the middle, pushing the fridge out of the safe range, while the
    // battery sags from comfortable to critical.
    let afternoon = [
        (5.0, 0.85),
        (5.4, 0.70),
        (6.8, 0.55),
        (8.6, 0.45),
        (9.1, 0.30),
        (6.0, 0.18),
        (4.8, 0.10),
    ];

    let readings: Vec<f32> = afternoon.iter().map(|&(reading, _)| reading).collect();
    let probe = FridgeProbe {
        readings: readings.into_iter(),
    };
    let mut node = Node::new(profile, probe, Cooler, link, CborCodec);

    // Each cycle the node reads, decides, switches the cooler, and publishes.
    let codec = CborCodec;
    for (step, (_, soc)) in afternoon.iter().enumerate() {
        let reaction = node.tick().await?;
        let reading: f32 = codec.decode(&gateway.recv().await?.expect("a reading").payload)?;

        let cooler = if reaction.actuator == Some(true) {
            "on "
        } else {
            "off"
        };
        let flag = if reaction.alert.is_some() {
            "  ALERT out of safe range"
        } else {
            ""
        };
        println!("step {step}: {reading:.1} C  cooler {cooler}{flag}");

        // Ask the profile how long to wait before the next sample at this charge.
        let (mode, wait) = node.schedule(*soc, false);
        let mode = match mode {
            PowerMode::Active => "active",
            PowerMode::Saver => "saver",
            PowerMode::Critical => "critical",
        };
        println!(
            "        battery {:.0}%, next sample in {}s ({mode})",
            soc * 100.0,
            wait.as_secs()
        );
    }

    // The profile that drove all of this is plain data: a community could ship it as
    // the manifest below, and a device would load it with `Profile::from_json`.
    println!("\nshareable manifest:\n{}", node.profile().to_json()?);

    println!("done");
    Ok(())
}
