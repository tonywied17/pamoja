# pamoja-serial::slip

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

SLIP framing, the Serial Line Internet Protocol of RFC 1055.

SLIP is the oldest and simplest way to put packets on a serial line. A single byte,
[`END`], marks the end of a packet. When that byte (or the escape byte [`ESC`]) appears
in the payload it is replaced by a two-byte escape sequence, so the delimiter can never
be mistaken for data. That is the whole protocol; its appeal is that it fits in a
handful of bytes of code on the smallest microcontroller.

The four byte values are fixed by RFC 1055: [`END`] is `0xC0`, [`ESC`] is `0xDB`, and
the escape sequences are `ESC` `ESC_END` for a literal `END` and `ESC` `ESC_ESC` for a
literal `ESC`.

## const `END`

The SLIP frame delimiter, RFC 1055 `END` (octal 300).

```rust
const END: u8
```

## const `ESC`

The SLIP escape byte, RFC 1055 `ESC` (octal 333).

```rust
const ESC: u8
```

## const `ESC_END`

The byte that follows [`ESC`] to encode a literal [`END`], RFC 1055 `ESC_END` (octal 334).

```rust
const ESC_END: u8
```

## const `ESC_ESC`

The byte that follows [`ESC`] to encode a literal [`ESC`], RFC 1055 `ESC_ESC` (octal 335).

```rust
const ESC_ESC: u8
```

## fn `max_encoded_len`

Returns an output length that is always large enough to hold the SLIP encoding of a
payload of `payload_len` bytes.

The worst case is a payload made entirely of [`END`] or [`ESC`] bytes, where every byte
becomes a two-byte escape sequence, plus the one trailing [`END`] delimiter.

**Arguments**

* `payload_len` - the length of the payload to be encoded.

**Returns**

The maximum number of bytes [`encode`] can write for that payload.

**Examples**

```
use pamoja_serial::slip;

assert_eq!(slip::max_encoded_len(10), 21);
```

```rust
const fn max_encoded_len(payload_len: usize) -> usize
```

## fn `encode`

Encodes a payload into a SLIP frame, terminated by an [`END`] byte.

Every [`END`] in the payload is written as `ESC` `ESC_END` and every [`ESC`] as `ESC`
`ESC_ESC`; all other bytes pass through unchanged. A single [`END`] is appended to mark
the end of the frame. A sender that wants RFC 1055's noise-flushing behaviour may
prepend an extra [`END`] of its own; [`decode`] and [`SlipDecoder`] ignore it.

**Arguments**

* `payload` - the bytes to frame.
* `output` - the buffer the frame is written into; size it with [`max_encoded_len`].

**Returns**

The number of bytes written to `output`.

**Errors**

Returns [`SerialError::BufferTooSmall`] if `output` cannot hold the whole frame.

**Examples**

```
use pamoja_serial::slip::{self, END, ESC, ESC_END};

// A payload containing the delimiter byte is escaped, never emitted raw.
let mut frame = [0u8; 8];
let n = slip::encode(&[0x01, END, 0x02], &mut frame)?;
assert_eq!(&frame[..n], &[0x01, ESC, ESC_END, 0x02, END]);
```

```rust
fn encode(payload: &[u8], output: &mut [u8]) -> Result <usize, SerialError>
```

## fn `decode`

Decodes a single SLIP frame, recovering the original payload.

A leading [`END`] (RFC 1055's flush byte) and any empty run before the payload are
skipped; decoding stops at the [`END`] that closes the frame, or at the end of the
slice if it carries no trailing delimiter.

**Arguments**

* `frame` - the framed bytes, with or without the trailing [`END`].
* `output` - the buffer the payload is written into; it never needs more room than
  `frame`.

**Returns**

The number of payload bytes written to `output`.

**Errors**

Returns [`SerialError::InvalidEscape`] if an [`ESC`] is followed by an unexpected byte,
[`SerialError::TruncatedFrame`] if the frame ends in the middle of an escape sequence,
and [`SerialError::BufferTooSmall`] if `output` cannot hold the payload.

**Examples**

```
use pamoja_serial::slip::{self, END};

let mut payload = [0u8; 4];
// A leading END is tolerated and the trailing one closes the frame.
let n = slip::decode(&[END, b'h', b'i', END], &mut payload)?;
assert_eq!(&payload[..n], b"hi");
```

```rust
fn decode(frame: &[u8], output: &mut [u8]) -> Result <usize, SerialError>
```

## struct `SlipDecoder`

A streaming SLIP decoder that reassembles whole frames from a serial byte stream.

A serial read returns whatever bytes have arrived, which is rarely a whole packet, so a
real receive loop feeds bytes in as they come and acts on each frame as it completes.
`SlipDecoder` buffers up to `N` payload bytes; [`push`](SlipDecoder::push) returns the
finished payload when an [`END`] closes the frame, and `None` while one is still being
assembled.

**Examples**

```
use pamoja_serial::slip::{SlipDecoder, END};

let mut decoder: SlipDecoder<32> = SlipDecoder::new();
let mut frames = 0;
// Two packets arrive back to back in one read.
for &byte in &[b'o', b'k', END, b'g', b'o', END] {
    if let Some(frame) = decoder.push(byte)? {
        frames += 1;
        assert!(frame == b"ok" || frame == b"go");
    }
}
assert_eq!(frames, 2);
```

### `SlipDecoder <N>::new`

Creates an empty decoder with room for an `N`-byte payload.

**Returns**

A decoder ready to receive the first byte.

```rust
const fn new() -> Self
```

### `SlipDecoder <N>::reset`

Discards any partly assembled frame, returning the decoder to its initial state.

```rust
fn reset(&mut self)
```

### `SlipDecoder <N>::push`

Feeds one byte from the stream into the decoder.

**Arguments**

* `byte` - the next byte received on the serial line.

**Returns**

`Some(payload)` when this byte completed a frame, or `None` while a frame is still
being assembled. An empty frame (a stray [`END`] with nothing buffered) is treated
as a flush and yields `None`.

**Errors**

Returns [`SerialError::InvalidEscape`] if an [`ESC`] is followed by an unexpected
byte, [`SerialError::TruncatedFrame`] if an [`END`] arrives mid-escape, and
[`SerialError::BufferTooSmall`] if the payload exceeds `N` bytes. After any error
the partial frame is discarded and the decoder resumes at the next byte.

```rust
fn push(&mut self, byte: u8) -> Result <Option <&[u8]>, SerialError>
```

