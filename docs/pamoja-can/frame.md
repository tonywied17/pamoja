# pamoja-can::frame

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

The CAN data frame and the length-to-DLC encoding it uses.

## fn `len_to_dlc`

Maps a data length to the data-length code that represents it.

Classic CAN and the first nine CAN-FD codes are the length itself, 0 through 8. Above
that CAN-FD jumps in steps, so a length between two steps rounds up to the next code.

**Arguments**

* `len` - the data length in bytes.

**Returns**

The 4-bit data-length code.

**Examples**

```
use pamoja_can::len_to_dlc;

assert_eq!(len_to_dlc(8), 8);
assert_eq!(len_to_dlc(12), 9);
assert_eq!(len_to_dlc(64), 15);
```

```rust
fn len_to_dlc(len: usize) -> u8
```

## fn `dlc_to_len`

Maps a data-length code to the number of bytes it represents.

**Arguments**

* `dlc` - the data-length code; only its low four bits are used.

**Returns**

The data length in bytes.

**Examples**

```
use pamoja_can::dlc_to_len;

assert_eq!(dlc_to_len(8), 8);
assert_eq!(dlc_to_len(15), 64);
```

```rust
fn dlc_to_len(dlc: u8) -> usize
```

## struct `Frame`

A CAN frame: an identifier and its data.

Holds a classic CAN 2.0 frame (up to 8 bytes), a CAN-FD frame (up to 64 bytes at the
discrete CAN-FD lengths), or a classic remote frame, which requests data and carries
none. The data lives in a fixed buffer, so building a frame never allocates.

**Examples**

```
use pamoja_can::{CanId, Frame};

let frame = Frame::new(CanId::standard(0x100), &[0x01, 0x02, 0x03]).unwrap();
assert_eq!(frame.data(), &[0x01, 0x02, 0x03]);
assert_eq!(frame.dlc(), 3);
assert!(!frame.is_fd());
```

### `Frame::new`

Builds a classic CAN 2.0 data frame.

**Arguments**

* `id` - the arbitration identifier.
* `data` - the payload, at most 8 bytes.

**Returns**

The frame.

**Errors**

Returns [`CanError::DataTooLong`] if `data` is longer than 8 bytes.

```rust
fn new(id: CanId, data: &[u8]) -> Result <Frame, CanError>
```

### `Frame::fd`

Builds a CAN-FD data frame.

**Arguments**

* `id` - the arbitration identifier.
* `data` - the payload, at one of the discrete CAN-FD lengths up to 64 bytes.

**Returns**

The frame.

**Errors**

Returns [`CanError::DataTooLong`] if `data` is longer than 64 bytes, or
[`CanError::InvalidFdLength`] if its length is not one CAN-FD can carry.

```rust
fn fd(id: CanId, data: &[u8]) -> Result <Frame, CanError>
```

### `Frame::remote`

Builds a classic remote frame, which requests data of a given length and carries
none.

**Arguments**

* `id` - the arbitration identifier.
* `len` - the data length being requested, clamped to 8 bytes.

**Returns**

The remote frame.

```rust
fn remote(id: CanId, len: usize) -> Frame
```

### `Frame::id`

Returns the arbitration identifier.

**Returns**

The identifier.

```rust
fn id(&self) -> CanId
```

### `Frame::data`

Returns the frame's data.

**Returns**

The payload bytes, or an empty slice for a remote frame.

```rust
fn data(&self) -> &[u8]
```

### `Frame::len`

Returns the data length: the payload length, or the requested length for a remote
frame.

**Returns**

The length in bytes.

```rust
fn len(&self) -> usize
```

### `Frame::is_empty`

Reports whether the frame carries no data.

**Returns**

`true` if the length is zero.

```rust
fn is_empty(&self) -> bool
```

### `Frame::dlc`

Returns the data-length code for this frame's length.

**Returns**

The 4-bit data-length code.

```rust
fn dlc(&self) -> u8
```

### `Frame::is_fd`

Reports whether this is a CAN-FD frame.

**Returns**

`true` for a CAN-FD frame.

```rust
fn is_fd(&self) -> bool
```

### `Frame::is_remote`

Reports whether this is a remote frame.

**Returns**

`true` for a remote frame.

```rust
fn is_remote(&self) -> bool
```

