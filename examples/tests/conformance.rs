//! Cross-crate conformance: the cold-chain scenario must hold at every seam.
//!
//! This drives the one scenario in `pamoja_examples::cold_chain` and asserts that each
//! crate it composes did its job, so a regression anywhere in the pipeline fails here.

use pamoja_examples::cold_chain;
use pamoja_profile::Alert;

#[tokio::test]
async fn cold_chain_pipeline_holds_end_to_end() {
    let outcome = cold_chain::run()
        .await
        .expect("the scenario runs to completion");

    assert!(!outcome.steps.is_empty(), "the run produced no steps");

    // Codec, loopback, and security round-trip: every reading reached the gateway
    // bit-for-bit and verified against the device's identity.
    for record in &outcome.steps {
        assert_eq!(
            record.sent.to_bits(),
            record.received.to_bits(),
            "step {}: the reading changed crossing the wire",
            record.step
        );
        assert!(
            record.verified,
            "step {}: a genuine frame failed to verify",
            record.step
        );
    }

    // Security: a frame altered after signing is rejected.
    assert!(outcome.tamper_rejected, "a tampered frame was accepted");

    // Profile control: the warming probe drove the fridge out of its safe range.
    assert!(
        outcome
            .steps
            .iter()
            .any(|record| matches!(record.reaction.alert, Some(Alert::OutOfRange { .. }))),
        "the excursion was never detected"
    );

    // Profile as data: the profile reloaded from its JSON manifest decides identically.
    assert!(
        outcome.reloaded_matches,
        "the reloaded profile decided differently"
    );

    // Audit: the genuine chain verifies and an edited one does not.
    assert!(
        outcome.genuine_chain_ok,
        "the genuine audit chain failed to verify"
    );
    assert!(
        !outcome.edited_chain_ok,
        "an edited audit chain still verified"
    );

    // Codec, metered link: the packed batch is smaller and within the quantizer precision.
    assert!(
        outcome.packed_bytes < outcome.raw_bytes,
        "packing did not shrink the batch ({} vs {} bytes)",
        outcome.packed_bytes,
        outcome.raw_bytes
    );
    assert!(
        outcome.max_quant_error <= outcome.quant_precision,
        "a reading exceeded the quantizer precision"
    );

    // Telemetry: every event counted, the excursion shipped, and routine detail was
    // dropped as the link cost rose.
    let expected_total = 2 * outcome.steps.len() as u32; // a tick and a reading per step
    assert_eq!(
        outcome.telemetry_total, expected_total,
        "telemetry miscounted events"
    );
    assert!(outcome.excursion_shipped, "an excursion event was dropped");
    assert!(
        outcome.error_events >= 1,
        "no excursion was reported at error level"
    );
    assert!(
        outcome.telemetry_emitted < outcome.telemetry_total,
        "the rising link cost dropped nothing"
    );
}
