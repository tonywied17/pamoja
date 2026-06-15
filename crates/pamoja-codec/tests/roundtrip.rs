//! Property tests: encoding a value and decoding it back returns the original,
//! for arbitrary inputs. Codecs sit on the untrusted-input boundary, so the
//! round-trip is exercised across a wide range of values rather than a few cases.

#![cfg(any(feature = "cbor", feature = "json"))]

use proptest::prelude::*;
use serde::{Deserialize, Serialize};

use pamoja_codec::Codec;

/// A representative payload: a string, an integer, a byte buffer, and a flag.
/// The fields avoid floating point so equality after a round trip is exact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Reading {
    sensor: String,
    sequence: u64,
    payload: Vec<u8>,
    ok: bool,
}

prop_compose! {
    fn any_reading()(
        sensor in ".*",
        sequence in any::<u64>(),
        payload in any::<Vec<u8>>(),
        ok in any::<bool>(),
    ) -> Reading {
        Reading { sensor, sequence, payload, ok }
    }
}

#[cfg(feature = "cbor")]
proptest! {
    #[test]
    fn cbor_round_trips(reading in any_reading()) {
        let codec = pamoja_codec::CborCodec;
        let encoded = codec.encode(&reading).expect("encode");
        let decoded: Reading = codec.decode(&encoded).expect("decode");
        prop_assert_eq!(decoded, reading);
    }
}

#[cfg(feature = "json")]
proptest! {
    #[test]
    fn json_round_trips(reading in any_reading()) {
        let codec = pamoja_codec::JsonCodec;
        let encoded = codec.encode(&reading).expect("encode");
        let decoded: Reading = codec.decode(&encoded).expect("decode");
        prop_assert_eq!(decoded, reading);
    }
}
