//! Conformance: offline-first store-and-forward must lose nothing over a flaky link.
//!
//! Drives the `pamoja_examples::degraded_link` scenario and asserts the backlog genuinely
//! built up while the link was down and that every reading still arrived, in order, so a
//! regression in store-and-forward or the degraded-link simulator fails here.

use pamoja_examples::degraded_link;

#[tokio::test]
async fn store_and_forward_survives_a_degraded_link() {
    let outcome = degraded_link::run()
        .await
        .expect("the scenario runs to completion");

    // The link genuinely degraded, so the node had to buffer: the test exercises the
    // offline path rather than a trivially perfect link.
    assert!(
        outcome.peak_backlog() > 0,
        "the backlog never built up, so the degraded link was not exercised"
    );

    // Nothing was lost: every reading reached the gateway.
    assert_eq!(
        outcome.received.len() as u64,
        outcome.total,
        "some readings never arrived"
    );

    // Nothing was reordered or duplicated: the readings arrived in sequence.
    let expected: Vec<u64> = (0..outcome.total).collect();
    assert_eq!(outcome.received, expected, "readings arrived out of order");
}
