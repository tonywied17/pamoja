# pamoja-serial::cobs

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

COBS framing, Consistent Overhead Byte Stuffing.

COBS (Cheshire and Baker, 1999) frames a packet by removing every zero byte from it, so
a single zero byte, the [`DELIMITER`], can mark the end of a frame and never be confused
with data. It encodes each run of up to 254 non-zero bytes as a length code followed by
the bytes themselves; a code of `0xFF` marks a full run of 254 non-zero bytes with no
zero after it, and a code from `0x01` to `0xFE` marks a shorter run that ended at a zero.

Its appeal over [SLIP](crate::slip) is the overhead: where SLIP can double a worst-case
payload, COBS adds at most one byte per 254 (see [`max_encoded_len`]), which is why
motor-control and robotics links that care about predictable framing cost prefer it.

[`encode`] produces the encoded block followed by the trailing zero delimiter, ready for
the wire; [`decode`] accepts a frame with or without that delimiter.

## const `DELIMITER`

The COBS frame delimiter: a single zero byte, the one value the encoding removes from
the payload so it can mark a frame boundary unambiguously.

```rust
const DELIMITER: u8
```

## fn `max_encoded_len`

Returns an output length that is always large enough to hold the COBS encoding of a
payload of `payload_len` bytes, including the trailing [`DELIMITER`].

COBS adds one code byte for every run of up to 254 bytes, plus the one delimiter, so the
overhead is bounded and small. This rounds the run count up and so may return one byte
more than the tightest possible encoding, which only ever over-allocates the buffer.

**Arguments**

* `payload_len` - the length of the payload to be encoded.

**Returns**

The maximum number of bytes [`encode`] can write for that payload.

**Examples**

```
use pamoja_serial::cobs;

// A short payload costs one code byte and one delimiter on top of the data.
assert_eq!(cobs::max_encoded_len(10), 12);
```

```rust
const fn max_encoded_len(payload_len: usize) -> usize
```

## fn `encode`

Encodes a payload into a COBS frame, terminated by the [`DELIMITER`] byte.

The encoded bytes are guaranteed to contain no zero except the trailing delimiter, so a
receiver can split a stream into frames on the zero byte alone.

**Arguments**

* `payload` - the bytes to frame.
* `output` - the buffer the frame is written into; size it with [`max_encoded_len`].

**Returns**

The number of bytes written to `output`, including the trailing [`DELIMITER`].

**Errors**

Returns [`SerialError::BufferTooSmall`] if `output` cannot hold the whole frame.

**Examples**

```
use pamoja_serial::cobs;

// The canonical example: a single zero byte encodes to 01 01, then the 00 delimiter.
let mut frame = [0u8; 4];
let n = cobs::encode(&[0x00], &mut frame)?;
assert_eq!(&frame[..n], &[0x01, 0x01, 0x00]);
```

```rust
fn encode(payload: &[u8], output: &mut [u8]) -> Result <usize, SerialError>
```

## fn `decode`

Decodes a single COBS frame, recovering the original payload.

Decoding stops at the [`DELIMITER`] that closes the frame, or at the end of the slice if
it carries no trailing delimiter.

**Arguments**

* `frame` - the encoded bytes, with or without the trailing [`DELIMITER`].
* `output` - the buffer the payload is written into; it never needs more room than
  `frame`.

**Returns**

The number of payload bytes written to `output`.

**Errors**

Returns [`SerialError::TruncatedFrame`] if a code byte claims more data than the frame
carries (which also catches a stray zero inside a run), and
[`SerialError::BufferTooSmall`] if `output` cannot hold the payload.

**Examples**

```
use pamoja_serial::cobs;

let mut payload = [0u8; 4];
let n = cobs::decode(&[0x03, 0x11, 0x22, 0x02, 0x33, 0x00], &mut payload)?;
assert_eq!(&payload[..n], &[0x11, 0x22, 0x00, 0x33]);
```

```rust
fn decode(frame: &[u8], output: &mut [u8]) -> Result <usize, SerialError>
```

## struct `CobsDecoder`

A streaming COBS decoder that reassembles whole frames from a serial byte stream.

Like [`SlipDecoder`](crate::slip::SlipDecoder), this is what a real serial receive loop
uses: it buffers up to `N` payload bytes and [`push`](CobsDecoder::push) returns the
finished payload when the zero [`DELIMITER`] closes a frame, or `None` while one is
still being assembled.

**Examples**

```
use pamoja_serial::cobs::{CobsDecoder, DELIMITER};

let mut decoder: CobsDecoder<32> = CobsDecoder::new();
// The encoding of the payload 11 22 00 33, followed by the delimiter.
let stream = [0x03, 0x11, 0x22, 0x02, 0x33, DELIMITER];
let mut got = None;
for &byte in &stream {
    if let Some(frame) = decoder.push(byte)? {
        got = Some(frame.to_vec());
    }
}
assert_eq!(got.as_deref(), Some(&[0x11, 0x22, 0x00, 0x33][..]));
```

### `CobsDecoder <N>::new`

Creates an empty decoder with room for an `N`-byte payload.

**Returns**

A decoder ready to receive the first byte.

```rust
const fn new() -> Self
```

### `CobsDecoder <N>::reset`

Discards any partly assembled frame, returning the decoder to its initial state.

```rust
fn reset(&mut self)
```

### `CobsDecoder <N>::push`

Feeds one byte from the stream into the decoder.

**Arguments**

* `byte` - the next byte received on the serial line.

**Returns**

`Some(payload)` when this byte's [`DELIMITER`] completed a frame, or `None` while a
frame is still being assembled.

**Errors**

Returns [`SerialError::TruncatedFrame`] if the delimiter arrives before a run's data
is complete, and [`SerialError::BufferTooSmall`] if the payload exceeds `N` bytes.
After any error the partial frame is discarded and the decoder resumes at the next
byte.

```rust
fn push(&mut self, byte: u8) -> Result <Option <&[u8]>, SerialError>
```

