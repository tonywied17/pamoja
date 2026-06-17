//! COBS framing, Consistent Overhead Byte Stuffing.
//!
//! COBS (Cheshire and Baker, 1999) frames a packet by removing every zero byte from it, so
//! a single zero byte, the [`DELIMITER`], can mark the end of a frame and never be confused
//! with data. It encodes each run of up to 254 non-zero bytes as a length code followed by
//! the bytes themselves; a code of `0xFF` marks a full run of 254 non-zero bytes with no
//! zero after it, and a code from `0x01` to `0xFE` marks a shorter run that ended at a zero.
//!
//! Its appeal over [SLIP](crate::slip) is the overhead: where SLIP can double a worst-case
//! payload, COBS adds at most one byte per 254 (see [`max_encoded_len`]), which is why
//! motor-control and robotics links that care about predictable framing cost prefer it.
//!
//! [`encode`] produces the encoded block followed by the trailing zero delimiter, ready for
//! the wire; [`decode`] accepts a frame with or without that delimiter.

use crate::SerialError;

/// The COBS frame delimiter: a single zero byte, the one value the encoding removes from
/// the payload so it can mark a frame boundary unambiguously.
pub const DELIMITER: u8 = 0x00;

/// The largest run of non-zero bytes a single COBS code can cover.
const MAX_RUN: usize = 254;

/// Returns an output length that is always large enough to hold the COBS encoding of a
/// payload of `payload_len` bytes, including the trailing [`DELIMITER`].
///
/// COBS adds one code byte for every run of up to 254 bytes, plus the one delimiter, so the
/// overhead is bounded and small. This rounds the run count up and so may return one byte
/// more than the tightest possible encoding, which only ever over-allocates the buffer.
///
/// # Arguments
///
/// * `payload_len` - the length of the payload to be encoded.
///
/// # Returns
///
/// The maximum number of bytes [`encode`] can write for that payload.
///
/// # Examples
///
/// ```
/// use pamoja_serial::cobs;
///
/// // A short payload costs one code byte and one delimiter on top of the data.
/// assert_eq!(cobs::max_encoded_len(10), 12);
/// ```
#[must_use]
pub const fn max_encoded_len(payload_len: usize) -> usize {
    payload_len + payload_len / MAX_RUN + 2
}

/// Encodes a payload into a COBS frame, terminated by the [`DELIMITER`] byte.
///
/// The encoded bytes are guaranteed to contain no zero except the trailing delimiter, so a
/// receiver can split a stream into frames on the zero byte alone.
///
/// # Arguments
///
/// * `payload` - the bytes to frame.
/// * `output` - the buffer the frame is written into; size it with [`max_encoded_len`].
///
/// # Returns
///
/// The number of bytes written to `output`, including the trailing [`DELIMITER`].
///
/// # Errors
///
/// Returns [`SerialError::BufferTooSmall`] if `output` cannot hold the whole frame.
///
/// # Examples
///
/// ```
/// use pamoja_serial::cobs;
///
/// // The canonical example: a single zero byte encodes to 01 01, then the 00 delimiter.
/// let mut frame = [0u8; 4];
/// let n = cobs::encode(&[0x00], &mut frame)?;
/// assert_eq!(&frame[..n], &[0x01, 0x01, 0x00]);
/// # Ok::<(), pamoja_serial::SerialError>(())
/// ```
pub fn encode(payload: &[u8], output: &mut [u8]) -> Result<usize, SerialError> {
    // Reserve output[0] for the first run's code byte; data follows from index 1.
    if output.len() < 2 {
        return Err(SerialError::BufferTooSmall);
    }
    let mut write = 1usize;
    // The index of the code byte for the run currently being built, or `None` once a full
    // 0xFF run has closed at the exact end of the payload (such a run owes no further code).
    let mut code_index: Option<usize> = Some(0);
    let mut code: u8 = 1;
    let n = payload.len();

    let mut i = 0usize;
    while i < n {
        let byte = payload[i];
        i += 1;
        if byte != DELIMITER {
            write_at(output, write, byte)?;
            write += 1;
            code += 1;
            if code == 0xFF {
                // A run of 254 non-zero bytes is full; close it with a 0xFF code.
                set_code(output, code_index, 0xFF);
                code = 1;
                if i < n {
                    // More payload follows, so start a fresh run.
                    reserve(output, &mut write, &mut code_index)?;
                } else {
                    // The payload ends exactly here; a 0xFF run carries no trailing zero,
                    // so there is no further code byte to write.
                    code_index = None;
                }
            }
        } else {
            // A zero ends the current run; its code records the run length.
            set_code(output, code_index, code);
            code = 1;
            reserve(output, &mut write, &mut code_index)?;
        }
    }
    set_code(output, code_index, code);
    write_at(output, write, DELIMITER)?;
    write += 1;
    Ok(write)
}

/// Decodes a single COBS frame, recovering the original payload.
///
/// Decoding stops at the [`DELIMITER`] that closes the frame, or at the end of the slice if
/// it carries no trailing delimiter.
///
/// # Arguments
///
/// * `frame` - the encoded bytes, with or without the trailing [`DELIMITER`].
/// * `output` - the buffer the payload is written into; it never needs more room than
///   `frame`.
///
/// # Returns
///
/// The number of payload bytes written to `output`.
///
/// # Errors
///
/// Returns [`SerialError::TruncatedFrame`] if a code byte claims more data than the frame
/// carries (which also catches a stray zero inside a run), and
/// [`SerialError::BufferTooSmall`] if `output` cannot hold the payload.
///
/// # Examples
///
/// ```
/// use pamoja_serial::cobs;
///
/// let mut payload = [0u8; 4];
/// let n = cobs::decode(&[0x03, 0x11, 0x22, 0x02, 0x33, 0x00], &mut payload)?;
/// assert_eq!(&payload[..n], &[0x11, 0x22, 0x00, 0x33]);
/// # Ok::<(), pamoja_serial::SerialError>(())
/// ```
pub fn decode(frame: &[u8], output: &mut [u8]) -> Result<usize, SerialError> {
    let mut write = 0usize;
    // Bytes still owed by the current run, and the code of the run just finished. Starting
    // `code` at 0xFF suppresses an implied zero before the very first run.
    let mut owed: u8 = 0;
    let mut code: u8 = 0xFF;
    for &byte in frame {
        if byte == DELIMITER {
            // A well-formed frame reaches the delimiter with the current run satisfied.
            if owed != 0 {
                return Err(SerialError::TruncatedFrame);
            }
            return Ok(write);
        }
        if owed != 0 {
            write_at(output, write, byte)?;
            write += 1;
            owed -= 1;
        } else {
            // `byte` is the next run's code. A finished run shorter than 0xFF ended at a
            // zero in the original data, so emit that implied zero before the new run.
            if code != 0xFF {
                write_at(output, write, DELIMITER)?;
                write += 1;
            }
            code = byte;
            owed = byte - 1;
        }
    }
    if owed != 0 {
        return Err(SerialError::TruncatedFrame);
    }
    Ok(write)
}

/// A streaming COBS decoder that reassembles whole frames from a serial byte stream.
///
/// Like [`SlipDecoder`](crate::slip::SlipDecoder), this is what a real serial receive loop
/// uses: it buffers up to `N` payload bytes and [`push`](CobsDecoder::push) returns the
/// finished payload when the zero [`DELIMITER`] closes a frame, or `None` while one is
/// still being assembled.
///
/// # Examples
///
/// ```
/// use pamoja_serial::cobs::{CobsDecoder, DELIMITER};
///
/// let mut decoder: CobsDecoder<32> = CobsDecoder::new();
/// // The encoding of the payload 11 22 00 33, followed by the delimiter.
/// let stream = [0x03, 0x11, 0x22, 0x02, 0x33, DELIMITER];
/// let mut got = None;
/// for &byte in &stream {
///     if let Some(frame) = decoder.push(byte)? {
///         got = Some(frame.to_vec());
///     }
/// }
/// assert_eq!(got.as_deref(), Some(&[0x11, 0x22, 0x00, 0x33][..]));
/// # Ok::<(), pamoja_serial::SerialError>(())
/// ```
#[derive(Debug)]
pub struct CobsDecoder<const N: usize> {
    buffer: [u8; N],
    len: usize,
    owed: u8,
    code: u8,
    complete: bool,
}

impl<const N: usize> CobsDecoder<N> {
    /// Creates an empty decoder with room for an `N`-byte payload.
    ///
    /// # Returns
    ///
    /// A decoder ready to receive the first byte.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; N],
            len: 0,
            owed: 0,
            code: 0xFF,
            complete: false,
        }
    }

    /// Discards any partly assembled frame, returning the decoder to its initial state.
    pub fn reset(&mut self) {
        self.len = 0;
        self.owed = 0;
        self.code = 0xFF;
        self.complete = false;
    }

    /// Feeds one byte from the stream into the decoder.
    ///
    /// # Arguments
    ///
    /// * `byte` - the next byte received on the serial line.
    ///
    /// # Returns
    ///
    /// `Some(payload)` when this byte's [`DELIMITER`] completed a frame, or `None` while a
    /// frame is still being assembled.
    ///
    /// # Errors
    ///
    /// Returns [`SerialError::TruncatedFrame`] if the delimiter arrives before a run's data
    /// is complete, and [`SerialError::BufferTooSmall`] if the payload exceeds `N` bytes.
    /// After any error the partial frame is discarded and the decoder resumes at the next
    /// byte.
    pub fn push(&mut self, byte: u8) -> Result<Option<&[u8]>, SerialError> {
        if self.complete {
            self.reset();
        }
        if byte == DELIMITER {
            if self.owed != 0 {
                self.reset();
                return Err(SerialError::TruncatedFrame);
            }
            self.complete = true;
            return Ok(Some(&self.buffer[..self.len]));
        }
        if self.owed != 0 {
            self.store(byte)?;
            self.owed -= 1;
        } else {
            if self.code != 0xFF {
                self.store(DELIMITER)?;
            }
            self.code = byte;
            self.owed = byte - 1;
        }
        Ok(None)
    }

    fn store(&mut self, byte: u8) -> Result<(), SerialError> {
        if self.len >= N {
            self.reset();
            return Err(SerialError::BufferTooSmall);
        }
        self.buffer[self.len] = byte;
        self.len += 1;
        Ok(())
    }
}

impl<const N: usize> Default for CobsDecoder<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Writes `byte` into `output[index]`, or reports the buffer is full.
fn write_at(output: &mut [u8], index: usize, byte: u8) -> Result<(), SerialError> {
    if index >= output.len() {
        return Err(SerialError::BufferTooSmall);
    }
    output[index] = byte;
    Ok(())
}

/// Records a run's length code at a previously reserved index, if one is pending.
///
/// The index was reserved by [`reserve`] (or is the initial code slot), so it is always in
/// bounds; a `None` index means a full 0xFF run already closed the encoding.
fn set_code(output: &mut [u8], code_index: Option<usize>, code: u8) {
    if let Some(index) = code_index {
        output[index] = code;
    }
}

/// Reserves the next code slot at `write`, advancing `write` past it, and points
/// `code_index` at it.
fn reserve(
    output: &[u8],
    write: &mut usize,
    code_index: &mut Option<usize>,
) -> Result<(), SerialError> {
    if *write >= output.len() {
        return Err(SerialError::BufferTooSmall);
    }
    *code_index = Some(*write);
    *write += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds the inclusive byte range `start..=end` as a vector.
    fn range(start: u8, end: u8) -> Vec<u8> {
        (start..=end).collect()
    }

    /// The canonical encoding examples from the COBS specification (Cheshire and Baker),
    /// each as (unencoded payload, encoded frame including the trailing zero delimiter).
    fn canonical_vectors() -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut v: Vec<(Vec<u8>, Vec<u8>)> = vec![
            (vec![0x00], vec![0x01, 0x01, 0x00]),
            (vec![0x00, 0x00], vec![0x01, 0x01, 0x01, 0x00]),
            (vec![0x00, 0x11, 0x00], vec![0x01, 0x02, 0x11, 0x01, 0x00]),
            (
                vec![0x11, 0x22, 0x00, 0x33],
                vec![0x03, 0x11, 0x22, 0x02, 0x33, 0x00],
            ),
            (
                vec![0x11, 0x22, 0x33, 0x44],
                vec![0x05, 0x11, 0x22, 0x33, 0x44, 0x00],
            ),
            (
                vec![0x11, 0x00, 0x00, 0x00],
                vec![0x02, 0x11, 0x01, 0x01, 0x01, 0x00],
            ),
        ];

        // Example 7: 254 non-zero bytes 01..FE encode to a single 0xFF run, no phantom code.
        let mut e7 = vec![0xFF];
        e7.extend(range(0x01, 0xFE));
        e7.push(0x00);
        v.push((range(0x01, 0xFE), e7));

        // Example 8: a leading zero, then 01..FE (255 bytes).
        let mut p8 = vec![0x00];
        p8.extend(range(0x01, 0xFE));
        let mut e8 = vec![0x01, 0xFF];
        e8.extend(range(0x01, 0xFE));
        e8.push(0x00);
        v.push((p8, e8));

        // Example 9: 01..FF (255 non-zero bytes) splits into a full run plus a trailing one.
        let mut e9 = vec![0xFF];
        e9.extend(range(0x01, 0xFE));
        e9.extend([0x02, 0xFF, 0x00]);
        v.push((range(0x01, 0xFF), e9));

        // Example 10: 02..FF then a trailing zero (255 bytes).
        let mut p10 = range(0x02, 0xFF);
        p10.push(0x00);
        let mut e10 = vec![0xFF];
        e10.extend(range(0x02, 0xFF));
        e10.extend([0x01, 0x01, 0x00]);
        v.push((p10, e10));

        // Example 11: 03..FF, a zero, then 01 (255 bytes).
        let mut p11 = range(0x03, 0xFF);
        p11.extend([0x00, 0x01]);
        let mut e11 = vec![0xFE];
        e11.extend(range(0x03, 0xFF));
        e11.extend([0x02, 0x01, 0x00]);
        v.push((p11, e11));

        v
    }

    #[test]
    fn encode_matches_the_canonical_vectors() {
        for (payload, expected) in canonical_vectors() {
            let mut out = vec![0u8; max_encoded_len(payload.len())];
            let n = encode(&payload, &mut out).unwrap();
            assert_eq!(&out[..n], &expected[..], "payload {payload:02x?}");
        }
    }

    #[test]
    fn decode_matches_the_canonical_vectors() {
        for (payload, encoded) in canonical_vectors() {
            let mut out = vec![0u8; payload.len()];
            let n = decode(&encoded, &mut out).unwrap();
            assert_eq!(&out[..n], &payload[..], "encoded {encoded:02x?}");
        }
    }

    #[test]
    fn the_encoding_never_contains_an_interior_zero() {
        for (payload, _) in canonical_vectors() {
            let mut out = vec![0u8; max_encoded_len(payload.len())];
            let n = encode(&payload, &mut out).unwrap();
            // Every byte except the final delimiter must be non-zero.
            assert!(out[..n - 1].iter().all(|&b| b != 0));
            assert_eq!(out[n - 1], DELIMITER);
        }
    }

    #[test]
    fn empty_payload_round_trips() {
        let mut frame = [0u8; 4];
        let n = encode(&[], &mut frame).unwrap();
        assert_eq!(&frame[..n], &[0x01, 0x00]);
        let mut out = [0u8; 4];
        let m = decode(&frame[..n], &mut out).unwrap();
        assert_eq!(m, 0);
    }

    #[test]
    fn round_trips_payloads_across_the_run_boundary() {
        // Lengths around 254/255 and a mix of zero and non-zero content stress the run
        // splitting and the implied-zero handling.
        for len in [0usize, 1, 2, 253, 254, 255, 256, 509, 510, 511] {
            for &fill in &[0x00u8, 0x41, 0xFF] {
                let payload = vec![fill; len];
                let mut frame = vec![0u8; max_encoded_len(len)];
                let n = encode(&payload, &mut frame).unwrap();
                let mut out = vec![0u8; len];
                let m = decode(&frame[..n], &mut out).unwrap();
                assert_eq!(&out[..m], &payload[..], "len {len} fill {fill:#04x}");
            }
        }
    }

    #[test]
    fn round_trips_a_mixed_payload_with_scattered_zeros() {
        let payload: Vec<u8> = (0..600u16).map(|i| (i % 7) as u8).collect();
        let mut frame = vec![0u8; max_encoded_len(payload.len())];
        let n = encode(&payload, &mut frame).unwrap();
        let mut out = vec![0u8; payload.len()];
        let m = decode(&frame[..n], &mut out).unwrap();
        assert_eq!(&out[..m], &payload[..]);
    }

    #[test]
    fn decode_tolerates_a_missing_trailing_delimiter() {
        // The same bytes as example 4 without the closing zero.
        let mut out = [0u8; 4];
        let n = decode(&[0x03, 0x11, 0x22, 0x02, 0x33], &mut out).unwrap();
        assert_eq!(&out[..n], &[0x11, 0x22, 0x00, 0x33]);
    }

    #[test]
    fn a_code_that_overruns_the_frame_is_truncated() {
        // Code 0x03 claims two data bytes but only one precedes the delimiter.
        let mut out = [0u8; 4];
        assert_eq!(
            decode(&[0x03, 0x11, 0x00], &mut out),
            Err(SerialError::TruncatedFrame)
        );
    }

    #[test]
    fn encode_reports_a_full_buffer() {
        let mut frame = [0u8; 3];
        assert_eq!(
            encode(&[0x11, 0x22, 0x33], &mut frame),
            Err(SerialError::BufferTooSmall)
        );
    }

    #[test]
    fn decode_reports_a_full_buffer() {
        let mut out = [0u8; 1];
        assert_eq!(
            decode(&[0x03, 0x11, 0x22, 0x00], &mut out),
            Err(SerialError::BufferTooSmall)
        );
    }

    #[test]
    fn streaming_decoder_matches_the_canonical_vectors() {
        for (payload, encoded) in canonical_vectors() {
            let mut decoder: CobsDecoder<512> = CobsDecoder::new();
            let mut produced = false;
            for &byte in &encoded {
                if let Some(frame) = decoder.push(byte).unwrap() {
                    assert_eq!(frame, &payload[..], "encoded {encoded:02x?}");
                    produced = true;
                }
            }
            assert!(produced, "no frame for {encoded:02x?}");
        }
    }

    #[test]
    fn streaming_decoder_reports_overflow_then_recovers() {
        let mut decoder: CobsDecoder<2> = CobsDecoder::new();
        // Encoding of three non-zero bytes overflows a two-byte buffer.
        assert!(decoder.push(0x04).unwrap().is_none());
        assert!(decoder.push(0x11).unwrap().is_none());
        assert!(decoder.push(0x22).unwrap().is_none());
        assert_eq!(decoder.push(0x33), Err(SerialError::BufferTooSmall));
        // A following frame (two bytes) decodes cleanly.
        assert!(decoder.push(0x03).unwrap().is_none());
        assert!(decoder.push(0xAA).unwrap().is_none());
        assert!(decoder.push(0xBB).unwrap().is_none());
        assert_eq!(decoder.push(DELIMITER).unwrap(), Some(&[0xAA, 0xBB][..]));
    }
}
