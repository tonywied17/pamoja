//! The whole SDK in one run: a cold-chain node from sensor to gateway over loopback.
//!
//! A simulated fridge probe warms past its safe range. Each reading is judged by the
//! device profile, encoded, signed and chained into a tamper-evident log, accounted for
//! by metered-link telemetry, and published to a gateway that verifies and decodes it.
//! Afterwards the run checks the integrity, metered-link, and shareable-profile
//! properties the cold-chain mission depends on. This is the same scenario the
//! conformance test asserts on, narrated. It composes `pamoja-sim`, `pamoja-profile`,
//! `pamoja-codec`, `pamoja-security`, `pamoja-audit`, `pamoja-telemetry`, and
//! `pamoja-loopback`, with no hardware.
//!
//! Run with: `cargo run -p pamoja-examples --example conformance`

use pamoja_examples::cold_chain;
use pamoja_profile::Alert;

#[tokio::main(flavor = "current_thread")]
async fn main() -> pamoja_core::Result<()> {
    let outcome = cold_chain::run().await?;

    println!(
        "device {} running '{}'\n",
        outcome.fingerprint, outcome.profile_name
    );

    for record in &outcome.steps {
        let cooler = match record.reaction.actuator {
            Some(true) => "on ",
            Some(false) => "off",
            None => "-  ",
        };
        let flag = if matches!(record.reaction.alert, Some(Alert::OutOfRange { .. })) {
            "  EXCURSION"
        } else {
            ""
        };
        let telemetry = if record.shipped { "ship" } else { "drop" };
        println!(
            "step {}: {:.2} C  cooler {cooler}  link {:?}/{telemetry}{flag}",
            record.step, record.received, record.link_cost
        );
    }

    let verified = outcome
        .steps
        .iter()
        .filter(|record| record.verified)
        .count();
    println!(
        "\nintegrity:   {verified}/{} frames verified, tampered frame rejected: {}",
        outcome.steps.len(),
        outcome.tamper_rejected
    );
    println!(
        "audit:       genuine chain verifies: {}, edited chain verifies: {}",
        outcome.genuine_chain_ok, outcome.edited_chain_ok
    );
    println!(
        "metered:     {} readings, raw {} B -> packed {} B, error <= {:.2} C",
        outcome.steps.len(),
        outcome.raw_bytes,
        outcome.packed_bytes,
        outcome.quant_precision
    );
    println!(
        "shareable:   profile reloaded from JSON decides identically: {}",
        outcome.reloaded_matches
    );
    println!(
        "telemetry:   {} events, {} shipped, {} dropped, snapshot packed into {} B",
        outcome.telemetry_total,
        outcome.telemetry_emitted,
        outcome.telemetry_dropped,
        outcome.snapshot_packed_bytes
    );

    Ok(())
}
