# pamoja-ros2::msg

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

CDR serialization and the geometry messages a robot is driven by.

ROS 2 and `rmw_zenoh` put messages on the wire as CDR, the OMG Common Data Representation. A
CDR stream opens with a four-byte encapsulation header naming the byte order, after which each
primitive is written in that byte order and aligned to its own size relative to the start of the
body. Getting the alignment padding wrong is the classic CDR bug, so the [`CdrWriter`] and
[`CdrReader`] handle it once, and the message types build on them. This slice covers the
little-endian encapsulation and the geometry messages used to command motion; more messages and
big-endian decoding arrive with the live bridge.

## struct `CdrWriter`

Writes primitives as little-endian CDR, handling alignment padding.

The writer starts with the little-endian encapsulation header; each write aligns the cursor to
the value's size (measured from the start of the body) before appending the bytes.

**Examples**

```
use pamoja_ros2::msg::CdrWriter;

let mut w = CdrWriter::new();
w.write_f64(1.0);
// Four-byte header plus eight bytes for the double.
assert_eq!(w.into_bytes().len(), 12);
```

### `CdrWriter::new`

Creates a writer primed with the little-endian CDR encapsulation header.

**Returns**

The writer.

```rust
fn new() -> Self
```

### `CdrWriter::write_i32`

Writes a 32-bit signed integer.

**Arguments**

* `value` - the integer to write.

```rust
fn write_i32(&mut self, value: i32)
```

### `CdrWriter::write_u32`

Writes a 32-bit unsigned integer.

**Arguments**

* `value` - the integer to write.

```rust
fn write_u32(&mut self, value: u32)
```

### `CdrWriter::write_f32`

Writes a 32-bit float.

**Arguments**

* `value` - the float to write.

```rust
fn write_f32(&mut self, value: f32)
```

### `CdrWriter::write_f64`

Writes a 64-bit float.

**Arguments**

* `value` - the float to write.

```rust
fn write_f64(&mut self, value: f64)
```

### `CdrWriter::into_bytes`

Consumes the writer and returns the encoded bytes, header included.

**Returns**

The CDR-encoded buffer.

```rust
fn into_bytes(self) -> Vec <u8>
```

## struct `CdrReader`

Reads primitives from a little-endian CDR buffer, handling alignment padding.

The reader checks the encapsulation header on construction and then mirrors [`CdrWriter`]'s
alignment, so a value written by the writer is read back identically.

### `CdrReader <'a>::new`

Creates a reader over a CDR buffer.

**Arguments**

* `data` - the CDR-encoded buffer, including the four-byte encapsulation header.

**Returns**

`Some(reader)` if `data` carries a classic little-endian CDR header; `None` otherwise,
including a buffer too short to hold a header or one declaring a byte order this reader does
not decode.

```rust
fn new(data: &'a [u8]) -> Option <Self>
```

### `CdrReader <'a>::read_i32`

Reads a 32-bit signed integer.

**Returns**

`Some(value)`, or `None` if the buffer is exhausted.

```rust
fn read_i32(&mut self) -> Option <i32>
```

### `CdrReader <'a>::read_u32`

Reads a 32-bit unsigned integer.

**Returns**

`Some(value)`, or `None` if the buffer is exhausted.

```rust
fn read_u32(&mut self) -> Option <u32>
```

### `CdrReader <'a>::read_f32`

Reads a 32-bit float.

**Returns**

`Some(value)`, or `None` if the buffer is exhausted.

```rust
fn read_f32(&mut self) -> Option <f32>
```

### `CdrReader <'a>::read_f64`

Reads a 64-bit float.

**Returns**

`Some(value)`, or `None` if the buffer is exhausted.

```rust
fn read_f64(&mut self) -> Option <f64>
```

## struct `Vector3`

A three-dimensional vector (`geometry_msgs/msg/Vector3`): three 64-bit floats.

Fields:

- `x: f64` - The x component.
- `y: f64` - The y component.
- `z: f64` - The z component.

### `Vector3::new`

Creates a vector from its components.

**Arguments**

* `x` - the x component.
* `y` - the y component.
* `z` - the z component.

**Returns**

The vector.

```rust
fn new(x: f64, y: f64, z: f64) -> Self
```

### `Vector3::encode`

Encodes the vector into a CDR writer.

**Arguments**

* `writer` - the writer to append to.

```rust
fn encode(&self, writer: &mut CdrWriter)
```

### `Vector3::decode`

Decodes a vector from a CDR reader.

**Arguments**

* `reader` - the reader to consume from.

**Returns**

`Some(vector)`, or `None` if the buffer is exhausted.

```rust
fn decode(reader: &mut CdrReader) -> Option <Self>
```

## struct `Twist`

A body velocity command (`geometry_msgs/msg/Twist`): a linear and an angular [`Vector3`].

This is the message a ROS 2 robot is driven by on `cmd_vel`, the natural target for the body
twists the `pamoja-kit` chassis and navigation helpers produce.

**Examples**

```
use pamoja_ros2::msg::{Twist, Vector3};

let cmd = Twist {
    linear: Vector3::new(0.5, 0.0, 0.0),
    angular: Vector3::new(0.0, 0.0, 0.2),
};
assert_eq!(Twist::from_cdr(&cmd.to_cdr()), Some(cmd));
```

Fields:

- `linear: Vector3` - The linear velocity, in metres per second.
- `angular: Vector3` - The angular velocity, in radians per second.

### `Twist::to_cdr`

Encodes the twist as a CDR message.

**Returns**

The CDR-encoded bytes, header included.

```rust
fn to_cdr(&self) -> Vec <u8>
```

### `Twist::from_cdr`

Decodes a twist from a CDR message.

**Arguments**

* `data` - the CDR-encoded bytes, header included.

**Returns**

`Some(twist)`, or `None` if the buffer is not a valid little-endian CDR twist.

```rust
fn from_cdr(data: &[u8]) -> Option <Self>
```

