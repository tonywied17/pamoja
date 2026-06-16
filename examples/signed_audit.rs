//! A tamper-evident cold-chain log: signed, hash-chained fridge readings.
//!
//! A vaccine-fridge node records a day of temperatures into an audit log, signing
//! each reading and chaining it to the one before, and persists the entry bytes the
//! way a field device writes to an SD card. An auditor later rebuilds the log from
//! storage and verifies it against the device's public identity: the genuine log
//! checks out, but quietly editing a single recorded reading to hide an excursion is
//! detected, because it breaks the chain. This is the audit trail the cold-chain
//! mission calls for. It composes `pamoja-audit`, `pamoja-security`, and
//! `pamoja-codec`, with no hardware.
//!
//! Run with: `cargo run -p pamoja-examples --example signed_audit`

use pamoja_audit::{verify_chain, AuditLog, Entry};
use pamoja_codec::{CborCodec, Codec};
use pamoja_core::Result;
use pamoja_security::DeviceIdentity;
use serde::{Deserialize, Serialize};

/// One hourly fridge reading.
#[derive(Debug, Serialize, Deserialize)]
struct Reading {
    hour: u8,
    celsius: f32,
}

/// Rebuilds entries from stored bytes and verifies the chain, reporting the result.
fn audit(label: &str, public: &pamoja_security::PublicIdentity, stored: &[Vec<u8>]) -> Result<()> {
    let entries: Vec<Entry> = stored
        .iter()
        .map(|bytes| Entry::from_bytes(bytes))
        .collect::<Result<_>>()?;
    match verify_chain(public, &entries) {
        Ok(()) => println!("{label}: {} readings verified, chain intact", entries.len()),
        Err(_) => println!("{label}: tampering detected, the log is not trustworthy"),
    }
    Ok(())
}

fn main() -> Result<()> {
    let codec = CborCodec;
    let device = DeviceIdentity::from_seed(&[9u8; 32]);
    let public = device.public();
    println!("recording for device {}", public.fingerprint());

    // The device records the day's readings, persisting each entry as it goes.
    let mut log = AuditLog::new(device);
    let day = [(6u8, 4.6f32), (9, 4.9), (12, 5.1), (15, 4.7)];
    let mut stored: Vec<Vec<u8>> = Vec::new();
    for (hour, celsius) in day {
        let reading = Reading { hour, celsius };
        let entry = log.append(&codec.encode(&reading)?);
        stored.push(entry.to_bytes());
    }

    // The genuine log verifies.
    audit("genuine log", &public, &stored)?;

    // Someone edits the noon reading to hide an excursion; the change is detected.
    let mut altered = stored.clone();
    *altered[2].last_mut().expect("a non-empty entry") ^= 0xff;
    audit("after editing one reading", &public, &altered)?;

    Ok(())
}
