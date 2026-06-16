//! Offline-first survives a flaky link: buffer, retry over a degraded link, lose nothing.
//!
//! A field node takes a reading each cycle and tries to forward its backlog over a
//! link that is reachable only part of the time. The degraded link rejects most
//! sends, but `drain_to` keeps the unsent records queued in order, so the backlog
//! rises while the link is down and falls when it returns. Once the link has cycled
//! enough, every reading has reached the gateway, in order, with nothing lost. This
//! composes `pamoja-sim`'s degraded link with `pamoja-sync` store-and-forward,
//! `pamoja-codec`, and `pamoja-loopback`, with no hardware and no broker.
//!
//! Run with: `cargo run -p pamoja-examples --example degraded_link`

use pamoja_codec::{CborCodec, Codec};
use pamoja_core::{Result, Store, Transport};
use pamoja_loopback::{LoopbackBroker, LoopbackTransport};
use pamoja_sim::DegradedLink;
use pamoja_sync::{drain_to, MemoryStore};
use serde::{Deserialize, Serialize};

/// A single temperature reading from a field sensor.
#[derive(Debug, Serialize, Deserialize)]
struct Reading {
    sequence: u64,
    celsius: f32,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let codec = CborCodec;
    let topic = "sensors/field-1/temperature";

    // A gateway listens on the broker; the node reaches it over a degraded link that
    // is up two sends, then down two, repeating.
    let broker = LoopbackBroker::new();
    let mut gateway = LoopbackTransport::new(broker.clone());
    gateway.connect().await?;
    gateway.subscribe("sensors/+/temperature").await?;

    let inner = LoopbackTransport::new(broker);
    let mut link = DegradedLink::new(inner).intermittent(2, 2);
    link.connect().await?;

    let mut outbox = MemoryStore::new();
    let total = 8;

    // Each cycle: take a reading, buffer it, and try to push the backlog out.
    for sequence in 0..total {
        let reading = Reading {
            sequence,
            celsius: 20.0 + sequence as f32,
        };
        outbox.append(&codec.encode(&reading)?).await?;

        // A degraded send leaves that record and the rest queued, in order, to retry.
        let _ = drain_to(&mut outbox, &mut link, topic).await;
        println!(
            "cycle {sequence}: {} reading(s) waiting on the link",
            outbox.len().await?
        );
    }

    // Keep retrying until the link has carried the whole backlog.
    while !outbox.is_empty().await? {
        let _ = drain_to(&mut outbox, &mut link, topic).await;
    }
    println!("backlog cleared");

    // Everything arrives at the gateway, in order, with nothing lost or reordered.
    for expected in 0..total {
        let message = gateway.recv().await?.expect("a forwarded reading");
        let reading: Reading = codec.decode(&message.payload)?;
        assert_eq!(reading.sequence, expected);
    }
    println!("gateway received all {total} readings in order: nothing lost");

    Ok(())
}
