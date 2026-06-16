//! Observability that degrades gracefully on a metered link.
//!
//! A node reports structured events at different severities. While the link is free
//! it ships everything; as the link becomes metered and then expensive, the reporter
//! raises its bar so only the events worth their bytes go out, while still counting
//! the rest so the aggregate picture stays complete. At the end it ships a compact,
//! delta-encoded snapshot of the counters instead of the raw event stream. Composes
//! `pamoja-telemetry` and `pamoja-codec`, with no hardware.
//!
//! Run with: `cargo run -p pamoja-examples --example telemetry`

use pamoja_codec::encode_deltas;
use pamoja_telemetry::{Event, Level, LinkCost, Reporter};

fn main() {
    let mut reporter = Reporter::new(Level::Trace);

    // A scripted run: the link cost at each point, and the event reported there.
    let run = [
        (LinkCost::Free, Event::debug("loop.tick")),
        (LinkCost::Free, Event::info("reading.ok").with_value(4.8)),
        (LinkCost::Metered, Event::debug("loop.tick")),
        (
            LinkCost::Metered,
            Event::warn("battery.low").with_value(0.18),
        ),
        (
            LinkCost::Expensive,
            Event::info("reading.ok").with_value(5.0),
        ),
        (LinkCost::Expensive, Event::error("link.lost")),
    ];

    for (cost, event) in run {
        reporter.adapt_to(cost);
        match reporter.record(event) {
            Some(shipped) => println!(
                "ship  {:?}  [{:?}] {} {:?}",
                cost, shipped.level, shipped.code, shipped.value
            ),
            None => println!("drop  {:?}  [{:?}] {}", cost, event.level, event.code),
        }
    }

    println!(
        "\nseen {} events, shipped {}, dropped {}",
        reporter.total(),
        reporter.emitted(),
        reporter.dropped()
    );

    // Ship the aggregate counters as a compact snapshot, not the raw stream.
    let snapshot = reporter.snapshot();
    let counters: Vec<i64> = snapshot
        .by_level
        .iter()
        .map(|&count| count as i64)
        .collect();
    let packed = encode_deltas(&counters);
    println!(
        "snapshot of {} per-level counts packed into {} bytes",
        counters.len(),
        packed.len()
    );
}
