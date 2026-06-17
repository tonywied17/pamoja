//! Offline-first survives a flaky link: buffer, retry over a degraded link, lose nothing.
//!
//! A field node takes a reading each cycle and tries to forward its backlog over a link
//! that is reachable only part of the time. The degraded link rejects most sends, but
//! store-and-forward keeps the unsent records queued in order, so the backlog rises while
//! the link is down and falls when it returns. Once the link has cycled enough, every
//! reading has reached the gateway, in order, with nothing lost. This is the same scenario
//! the conformance test asserts on, narrated. It composes `pamoja-sim`'s degraded link
//! with `pamoja-sync` store-and-forward, `pamoja-codec`, and `pamoja-loopback`, with no
//! hardware and no broker.
//!
//! Run with: `cargo run -p pamoja-examples --example degraded_link`

use pamoja_examples::degraded_link;

#[tokio::main(flavor = "current_thread")]
async fn main() -> pamoja_core::Result<()> {
    let outcome = degraded_link::run().await?;

    for (cycle, backlog) in outcome.backlog_per_cycle.iter().enumerate() {
        println!("cycle {cycle}: {backlog} reading(s) waiting on the link");
    }

    println!(
        "\nbacklog peaked at {} while the link was down, cleared after {} drain attempts",
        outcome.peak_backlog(),
        outcome.drain_attempts
    );
    println!(
        "gateway received all {} readings in order: {:?}",
        outcome.total, outcome.received
    );

    Ok(())
}
