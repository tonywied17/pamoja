//! MAVLink 2 message signing: the signature a sender appends and the check a receiver
//! makes, so a ground station can trust that a command came from the vehicle it expects
//! and was not replayed.
//!
//! The scheme follows the MAVLink reference exactly. A signed frame carries a 13-byte
//! block after its checksum: a one-byte link id, a 48-bit timestamp, and a 48-bit
//! signature. The signature is the first six bytes of
//! `SHA-256(secret_key ++ header ++ payload ++ checksum ++ link_id ++ timestamp)`, where
//! the header includes the start marker. The timestamp is in 10-microsecond units since
//! 1 January 2015 GMT and must increase, which is what stops a captured frame from being
//! replayed to re-arm a vehicle or re-trigger an actuator.
//!
//! [`Signer`] stamps and signs outgoing frames; [`Verifier`] checks incoming ones,
//! tracking a timestamp per `(system, component, link)` stream so an old or repeated
//! frame is rejected. SHA-256 is the one primitive borrowed (from `sha2`), as in
//! [`pamoja-session`](https://docs.rs/pamoja-session); everything else is built here.

use sha2::{Digest, Sha256};

use crate::error::{MavlinkError, Result};
use crate::frame::{Frame, Header, IFLAG_SIGNED};

/// The length of a signing secret key, in bytes.
pub const KEY_LEN: usize = 32;

/// The default replay window: one minute, in 10-microsecond ticks.
///
/// A frame from a stream not seen before is rejected if its timestamp is more than this
/// far behind the newest timestamp the verifier has accepted.
pub const DEFAULT_TIMESTAMP_WINDOW: u64 = 6_000_000;

/// The seconds between the Unix epoch and the MAVLink signing epoch (1 January 2015 GMT).
pub const MAVLINK_EPOCH_OFFSET_SECS: u64 = 1_420_070_400;

// The number of per-stream timestamps a verifier remembers at once.
const MAX_STREAMS: usize = 16;

// Computes the 48-bit signature over a frame: the first six bytes of the SHA-256 of the
// key, the frame's header-payload-checksum region, and the link id and timestamp.
fn signature_48(key: &[u8; KEY_LEN], signed_region: &[u8], link_and_timestamp: &[u8]) -> [u8; 6] {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(signed_region);
    hasher.update(link_and_timestamp);
    let digest = hasher.finalize();
    let mut out = [0u8; 6];
    out.copy_from_slice(&digest[..6]);
    out
}

/// Converts a wall-clock time to a MAVLink signing timestamp.
///
/// # Arguments
///
/// * `unix_micros` - microseconds since the Unix epoch, as a field clock (RTC or GPS)
///   would report.
///
/// # Returns
///
/// The timestamp in 10-microsecond ticks since the MAVLink epoch, or `0` if the time is
/// before that epoch.
pub fn timestamp_from_unix_micros(unix_micros: u64) -> u64 {
    let epoch_micros = MAVLINK_EPOCH_OFFSET_SECS * 1_000_000;
    unix_micros.saturating_sub(epoch_micros) / 10
}

/// Signs outgoing v2 frames with a shared key.
///
/// A signer holds the key, the link id it stamps, and a monotonically increasing
/// timestamp. Seed the timestamp from a field clock with
/// [`timestamp_from_unix_micros`] where one is available; without a clock, any strictly
/// increasing seed works, since the receiver only requires that timestamps rise.
#[derive(Clone)]
pub struct Signer {
    key: [u8; KEY_LEN],
    link_id: u8,
    timestamp: u64,
}

impl Signer {
    /// Creates a signer for a key, a link id, and a starting timestamp.
    ///
    /// # Arguments
    ///
    /// * `key` - the 32-byte shared secret.
    /// * `link_id` - the id of the link this signer stamps on its frames.
    /// * `timestamp` - the first timestamp to use, in 10-microsecond ticks since the
    ///   MAVLink epoch.
    ///
    /// # Returns
    ///
    /// The signer.
    pub fn new(key: [u8; KEY_LEN], link_id: u8, timestamp: u64) -> Self {
        Signer {
            key,
            link_id,
            timestamp,
        }
    }

    /// Builds and signs a v2 frame for a message.
    ///
    /// The frame is assembled with the signed flag set so the flag is covered by the
    /// checksum, then the signature block is filled and the signer's timestamp advanced.
    ///
    /// # Arguments
    ///
    /// * `header` - the addressing fields to stamp on the frame.
    /// * `msgid` - the 24-bit message id.
    /// * `payload` - the serialized message payload.
    /// * `crc_extra` - the `CRC_EXTRA` seed for `msgid`.
    ///
    /// # Returns
    ///
    /// The signed frame, ready to send.
    ///
    /// # Errors
    ///
    /// Returns [`MavlinkError::PayloadTooLong`] if the payload does not fit a frame.
    pub fn sign(
        &mut self,
        header: Header,
        msgid: u32,
        payload: &[u8],
        crc_extra: u8,
    ) -> Result<Frame> {
        let mut frame = Frame::assemble_v2(header, msgid, payload, crc_extra, IFLAG_SIGNED)?;
        let timestamp = self.timestamp;
        {
            let block = frame.signature_mut();
            block[0] = self.link_id;
            block[1..7].copy_from_slice(&timestamp.to_le_bytes()[..6]);
        }
        let mac = signature_48(
            &self.key,
            frame.signed_region(),
            &frame.signature().expect("just assembled as signed")[..7],
        );
        frame.signature_mut()[7..13].copy_from_slice(&mac);
        self.timestamp = self.timestamp.wrapping_add(1);
        Ok(frame)
    }

    /// Returns the link id this signer stamps.
    ///
    /// # Returns
    ///
    /// The link id.
    pub fn link_id(&self) -> u8 {
        self.link_id
    }
}

// One remembered stream: the newest timestamp accepted for a (system, component, link).
#[derive(Clone, Copy)]
struct Stream {
    system_id: u8,
    component_id: u8,
    link_id: u8,
    timestamp: u64,
    used: bool,
}

/// Verifies signed v2 frames against a shared key, rejecting forged and replayed frames.
///
/// A verifier recomputes each frame's signature and rejects it unless it matches. It also
/// enforces freshness: a frame from a stream it has seen must carry a strictly newer
/// timestamp than the last, and a frame from a new stream must not be more than the
/// replay window behind the newest timestamp seen, so a recording of an old frame cannot
/// be re-injected.
#[derive(Clone)]
pub struct Verifier {
    key: [u8; KEY_LEN],
    window: u64,
    newest: u64,
    streams: [Stream; MAX_STREAMS],
}

impl Verifier {
    /// Creates a verifier for a key, using the default replay window.
    ///
    /// # Arguments
    ///
    /// * `key` - the 32-byte shared secret.
    ///
    /// # Returns
    ///
    /// The verifier.
    pub fn new(key: [u8; KEY_LEN]) -> Self {
        Verifier {
            key,
            window: DEFAULT_TIMESTAMP_WINDOW,
            newest: 0,
            streams: [Stream {
                system_id: 0,
                component_id: 0,
                link_id: 0,
                timestamp: 0,
                used: false,
            }; MAX_STREAMS],
        }
    }

    /// Sets the replay window, in 10-microsecond ticks.
    ///
    /// # Arguments
    ///
    /// * `window` - how far behind the newest accepted timestamp a frame from a new
    ///   stream may be.
    ///
    /// # Returns
    ///
    /// The verifier, for chaining.
    pub fn with_window(mut self, window: u64) -> Self {
        self.window = window;
        self
    }

    /// Verifies a signed frame's signature and freshness.
    ///
    /// On success, the frame's stream timestamp is recorded so a later replay of the same
    /// or an older frame is rejected.
    ///
    /// # Arguments
    ///
    /// * `frame` - the parsed frame to check.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the frame is authentic and fresh.
    ///
    /// # Errors
    ///
    /// Returns [`MavlinkError::Unsigned`] if the frame is not signed,
    /// [`MavlinkError::BadSignature`] if the signature does not match the key, or
    /// [`MavlinkError::ReplayedTimestamp`] if the timestamp is not fresh.
    pub fn verify(&mut self, frame: &Frame) -> Result<()> {
        let block = frame.signature().ok_or(MavlinkError::Unsigned)?;
        let expected = signature_48(&self.key, frame.signed_region(), &block[..7]);
        if expected != block[7..13] {
            return Err(MavlinkError::BadSignature);
        }

        let link_id = block[0];
        let mut timestamp_bytes = [0u8; 8];
        timestamp_bytes[..6].copy_from_slice(&block[1..7]);
        let timestamp = u64::from_le_bytes(timestamp_bytes);

        let system_id = frame.system_id();
        let component_id = frame.component_id();
        match self.find_stream(system_id, component_id, link_id) {
            Some(index) => {
                if timestamp <= self.streams[index].timestamp {
                    return Err(MavlinkError::ReplayedTimestamp);
                }
                self.streams[index].timestamp = timestamp;
            }
            None => {
                if timestamp + self.window < self.newest {
                    return Err(MavlinkError::ReplayedTimestamp);
                }
                self.remember(system_id, component_id, link_id, timestamp);
            }
        }
        if timestamp > self.newest {
            self.newest = timestamp;
        }
        Ok(())
    }

    fn find_stream(&self, system_id: u8, component_id: u8, link_id: u8) -> Option<usize> {
        self.streams.iter().position(|stream| {
            stream.used
                && stream.system_id == system_id
                && stream.component_id == component_id
                && stream.link_id == link_id
        })
    }

    // Records a new stream's timestamp, evicting the stream with the oldest timestamp
    // when the table is full so a busy link cannot crowd out freshness tracking forever.
    fn remember(&mut self, system_id: u8, component_id: u8, link_id: u8, timestamp: u64) {
        let slot = self
            .streams
            .iter()
            .position(|stream| !stream.used)
            .unwrap_or_else(|| {
                self.streams
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, stream)| stream.timestamp)
                    .map(|(index, _)| index)
                    .unwrap_or(0)
            });
        self.streams[slot] = Stream {
            system_id,
            component_id,
            link_id,
            timestamp,
            used: true,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: [u8; KEY_LEN] = [0x42; KEY_LEN];

    fn signed_heartbeat(signer: &mut Signer, timestamp_seq: u8) -> Frame {
        let header = Header::new(1, 1, timestamp_seq);
        // A HEARTBEAT payload; CRC_EXTRA 50.
        signer
            .sign(header, 0, &[0, 0, 0, 0, 6, 8, 0, 3, 3], 50)
            .unwrap()
    }

    #[test]
    fn sha256_primitive_matches_the_nist_vector() {
        // SHA-256("abc"), the canonical FIPS-180 example, anchoring the primitive the
        // signature is built on.
        let mut hasher = Sha256::new();
        hasher.update(b"abc");
        let digest = hasher.finalize();
        let expected = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];
        assert_eq!(digest[..], expected[..]);
    }

    #[test]
    fn the_signature_block_is_laid_out_as_the_spec_requires() {
        let mut signer = Signer::new(KEY, 0x07, 0x0000_1122_3344_5566);
        let frame = signed_heartbeat(&mut signer, 0);
        let block = frame.signature().expect("signed");
        assert_eq!(block[0], 0x07); // link id
                                    // 48-bit timestamp, little-endian: the low six bytes of the seed.
        assert_eq!(&block[1..7], &[0x66, 0x55, 0x44, 0x33, 0x22, 0x11]);
        assert!(frame.is_signed());
    }

    #[test]
    fn a_signed_frame_verifies() {
        let mut signer = Signer::new(KEY, 1, 1_000_000);
        let mut verifier = Verifier::new(KEY);
        let frame = signed_heartbeat(&mut signer, 0);
        assert!(verifier.verify(&frame).is_ok());
    }

    #[test]
    fn a_tampered_frame_fails_verification() {
        let mut signer = Signer::new(KEY, 1, 1_000_000);
        let mut verifier = Verifier::new(KEY);
        let frame = signed_heartbeat(&mut signer, 0);
        let mut bytes = frame.as_bytes().to_vec();
        // Flip the last signature byte: the checksum does not cover it, so the frame still
        // parses, but the recomputed signature no longer matches.
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        let tampered = Frame::parse(&bytes, 50).unwrap();
        assert_eq!(verifier.verify(&tampered), Err(MavlinkError::BadSignature));
    }

    #[test]
    fn the_wrong_key_fails_verification() {
        let mut signer = Signer::new(KEY, 1, 1_000_000);
        let mut verifier = Verifier::new([0x99; KEY_LEN]);
        let frame = signed_heartbeat(&mut signer, 0);
        assert_eq!(verifier.verify(&frame), Err(MavlinkError::BadSignature));
    }

    #[test]
    fn an_unsigned_frame_is_rejected_by_a_verifier() {
        let header = Header::new(1, 1, 0);
        let frame = Frame::encode_v2(header, 0, &[0, 0, 0, 0, 6, 8, 0, 3, 3], 50).unwrap();
        let mut verifier = Verifier::new(KEY);
        assert_eq!(verifier.verify(&frame), Err(MavlinkError::Unsigned));
    }

    #[test]
    fn a_replayed_frame_is_rejected() {
        let mut signer = Signer::new(KEY, 1, 1_000_000);
        let mut verifier = Verifier::new(KEY);
        let frame = signed_heartbeat(&mut signer, 0);
        assert!(verifier.verify(&frame).is_ok());
        // The very same frame, replayed, carries a timestamp no newer than the last.
        assert_eq!(
            verifier.verify(&frame),
            Err(MavlinkError::ReplayedTimestamp)
        );
    }

    #[test]
    fn timestamps_must_increase_on_a_stream() {
        let mut verifier = Verifier::new(KEY);
        let mut newer = Signer::new(KEY, 1, 100);
        let mut older = Signer::new(KEY, 1, 50);
        let new_frame = signed_heartbeat(&mut newer, 0);
        let old_frame = signed_heartbeat(&mut older, 1);
        assert!(verifier.verify(&new_frame).is_ok());
        // A frame on the same stream with an older timestamp is a replay.
        assert_eq!(
            verifier.verify(&old_frame),
            Err(MavlinkError::ReplayedTimestamp)
        );
    }

    #[test]
    fn timestamp_conversion_uses_the_mavlink_epoch() {
        // Exactly at the MAVLink epoch, the timestamp is zero.
        assert_eq!(
            timestamp_from_unix_micros(MAVLINK_EPOCH_OFFSET_SECS * 1_000_000),
            0
        );
        // One second later is 100,000 ten-microsecond ticks.
        assert_eq!(
            timestamp_from_unix_micros((MAVLINK_EPOCH_OFFSET_SECS + 1) * 1_000_000),
            100_000
        );
    }
}
