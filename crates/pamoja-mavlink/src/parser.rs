//! A streaming frame parser that turns a byte stream into whole frames.
//!
//! A link delivers bytes, not frames: a serial port hands over whatever arrived since the
//! last read, and even a UDP datagram can carry several frames back to back. This parser
//! bridges that gap. Bytes are fed in as they arrive, and a complete, checksum-verified
//! [`Frame`] is returned as soon as one is recognized. A stray byte or a frame mangled in
//! transit makes the parser resynchronize on the next start marker rather than wedge, so a
//! noisy link recovers on its own. It holds a single fixed buffer, so it runs unchanged on
//! a microcontroller.

use crate::frame::{Frame, MAGIC_V1, MAGIC_V2, MAX_FRAME};

// What to do given the bytes buffered so far.
enum Decision {
    // Not enough bytes yet to decide; keep accumulating.
    NeedMore,
    // The buffer does not start a valid frame; drop the leading byte and re-examine.
    Resync,
    // A complete, valid frame occupies the first `usize` bytes of the buffer.
    Frame(usize),
}

/// Accumulates bytes from a link and emits complete frames.
///
/// Feed bytes with [`push_byte`](Parser::push_byte); each call returns a [`Frame`] on the
/// byte that completes one. Resolving a message id to its `CRC_EXTRA` is left to the
/// caller, so the parser works against any dialect.
#[derive(Clone)]
pub struct Parser {
    buf: [u8; MAX_FRAME],
    len: usize,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    /// Creates an empty parser.
    ///
    /// # Returns
    ///
    /// A parser with nothing buffered.
    pub fn new() -> Self {
        Parser {
            buf: [0u8; MAX_FRAME],
            len: 0,
        }
    }

    /// Feeds one byte, returning a frame if this byte completes a valid one.
    ///
    /// A frame whose checksum fails, or whose message id `crc_extra_for` does not
    /// recognize, is discarded and the parser resynchronizes on the next start marker.
    ///
    /// # Arguments
    ///
    /// * `byte` - the next byte from the link.
    /// * `crc_extra_for` - resolves a message id to its `CRC_EXTRA`, or `None` if unknown.
    ///
    /// # Returns
    ///
    /// The completed frame, or [`None`] if more bytes are needed.
    pub fn push_byte<F>(&mut self, byte: u8, crc_extra_for: &F) -> Option<Frame>
    where
        F: Fn(u32) -> Option<u8>,
    {
        if self.len >= MAX_FRAME {
            // A frame can never exceed the buffer; an overlong run of bytes that never
            // resolved is noise, so start fresh.
            self.len = 0;
        }
        self.buf[self.len] = byte;
        self.len += 1;

        loop {
            match self.decide(crc_extra_for) {
                Decision::NeedMore => return None,
                Decision::Resync => {
                    self.drop_front(1);
                    if self.len == 0 {
                        return None;
                    }
                }
                Decision::Frame(total) => {
                    let frame = Frame::parse_with(&self.buf[..total], crc_extra_for)
                        .expect("decide only reports a frame the checksum accepted");
                    self.drop_front(total);
                    return Some(frame);
                }
            }
        }
    }

    fn decide<F>(&self, crc_extra_for: &F) -> Decision
    where
        F: Fn(u32) -> Option<u8>,
    {
        if self.len == 0 {
            return Decision::NeedMore;
        }
        let header_len = match self.buf[0] {
            MAGIC_V1 => 6,
            MAGIC_V2 => 10,
            _ => return Decision::Resync,
        };
        // The length byte, and for v2 the incompat flags, decide the total frame size.
        let need_for_size = if self.buf[0] == MAGIC_V2 { 3 } else { 2 };
        if self.len < need_for_size {
            return Decision::NeedMore;
        }
        let plen = self.buf[1] as usize;
        let signed = self.buf[0] == MAGIC_V2 && self.buf[2] & 0x01 != 0;
        let total = header_len + plen + 2 + if signed { 13 } else { 0 };
        if self.len < total {
            return Decision::NeedMore;
        }
        match Frame::parse_with(&self.buf[..total], crc_extra_for) {
            Ok(_) => Decision::Frame(total),
            Err(_) => Decision::Resync,
        }
    }

    fn drop_front(&mut self, n: usize) {
        let n = n.min(self.len);
        self.buf.copy_within(n..self.len, 0);
        self.len -= n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Frame, Header};

    // The common dialect's CRC_EXTRA for HEARTBEAT.
    fn heartbeat_crc(id: u32) -> Option<u8> {
        (id == 0).then_some(50)
    }

    fn heartbeat_frame(seq: u8) -> Frame {
        Frame::encode_v2(Header::new(1, 1, seq), 0, &[0, 0, 0, 0, 6, 8, 0, 3, 3], 50).unwrap()
    }

    fn feed(parser: &mut Parser, bytes: &[u8]) -> Vec<Frame> {
        let mut out = Vec::new();
        for &byte in bytes {
            if let Some(frame) = parser.push_byte(byte, &heartbeat_crc) {
                out.push(frame);
            }
        }
        out
    }

    #[test]
    fn a_whole_frame_is_parsed() {
        let mut parser = Parser::new();
        let frame = heartbeat_frame(1);
        let parsed = feed(&mut parser, frame.as_bytes());
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].sequence(), 1);
    }

    #[test]
    fn a_frame_split_across_feeds_is_parsed() {
        let mut parser = Parser::new();
        let frame = heartbeat_frame(2);
        let bytes = frame.as_bytes();
        let (head, tail) = bytes.split_at(4);
        assert!(feed(&mut parser, head).is_empty());
        let parsed = feed(&mut parser, tail);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].sequence(), 2);
    }

    #[test]
    fn garbage_before_a_frame_is_skipped() {
        let mut parser = Parser::new();
        let frame = heartbeat_frame(3);
        let mut stream = vec![0x00, 0xFF, 0x12, 0x34]; // leading noise, no start marker
        stream.extend_from_slice(frame.as_bytes());
        let parsed = feed(&mut parser, &stream);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].sequence(), 3);
    }

    #[test]
    fn two_back_to_back_frames_both_emit() {
        let mut parser = Parser::new();
        let mut stream = Vec::new();
        stream.extend_from_slice(heartbeat_frame(10).as_bytes());
        stream.extend_from_slice(heartbeat_frame(11).as_bytes());
        let parsed = feed(&mut parser, &stream);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].sequence(), 10);
        assert_eq!(parsed[1].sequence(), 11);
    }

    #[test]
    fn a_corrupt_frame_is_dropped_and_the_next_recovers() {
        let mut parser = Parser::new();
        let mut corrupt = heartbeat_frame(20).as_bytes().to_vec();
        let last = corrupt.len() - 1;
        corrupt[last] ^= 0xFF; // break the checksum
        let mut stream = corrupt;
        stream.extend_from_slice(heartbeat_frame(21).as_bytes());
        let parsed = feed(&mut parser, &stream);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].sequence(), 21);
    }
}
