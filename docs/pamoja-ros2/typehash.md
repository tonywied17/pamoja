# pamoja-ros2::typehash

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Message type identity: the DDS type name and the RIHS01 type hash.

Two peers only exchange a message if they agree on its type. ROS 2 pins that agreement two ways:
a DDS type name derived from the interface (`std_msgs/msg/String` becomes
`std_msgs::msg::dds_::String_`), and a structural type hash (REP-2011's RIHS01, a SHA-256 over
the type's description, written `RIHS01_` followed by 64 hex digits). This module derives the
type name and parses and formats the hash. Computing the hash from a type description is part of
the live bridge, where it is checked against the value `rosidl` emits.

## struct `TypeHash`

A parsed ROS 2 type hash in the RIHS01 scheme (REP-2011).

The string form is `RIHS01_` followed by 64 lowercase hex digits, the version prefix plus a
SHA-256 digest of the type's description.

**Examples**

```
use pamoja_ros2::typehash::TypeHash;

// The published hash of std_msgs/msg/String round-trips through parse and display.
let text = "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18";
let hash = TypeHash::parse(text).unwrap();
assert_eq!(hash.to_string(), text);
```

### `TypeHash::parse`

Parses a RIHS01 hash string.

**Arguments**

* `text` - the candidate hash, expected as `RIHS01_` plus 64 lowercase hex digits.

**Returns**

`Some(hash)` if `text` is a well-formed RIHS01 string, otherwise `None`.

```rust
fn parse(text: &str) -> Option <Self>
```

### `TypeHash::digest`

Returns the raw 32-byte digest.

**Returns**

The SHA-256 digest carried by the hash.

```rust
fn digest(&self) -> [u8 ; HASH_LEN]
```

## fn `dds_type_name`

Derives the DDS type name from a ROS 2 interface type.

**Arguments**

* `ros_type` - the interface type as `package/namespace/Type`, for example `std_msgs/msg/String`.

**Returns**

`Some(dds_name)` such as `std_msgs::msg::dds_::String_`, joining the parts with `::`, inserting
the `dds_` namespace, and suffixing the type with `_`; `None` if `ros_type` is not three
non-empty `/`-separated parts.

**Examples**

```
use pamoja_ros2::typehash::dds_type_name;

assert_eq!(dds_type_name("std_msgs/msg/String").as_deref(), Some("std_msgs::msg::dds_::String_"));
assert_eq!(
    dds_type_name("example_interfaces/srv/AddTwoInts").as_deref(),
    Some("example_interfaces::srv::dds_::AddTwoInts_"),
);
assert_eq!(dds_type_name("std_msgs/String"), None); // missing the namespace part
```

```rust
fn dds_type_name(ros_type: &str) -> Option <String>
```

