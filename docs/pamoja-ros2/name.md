# pamoja-ros2::name

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

ROS 2 topic and service names: validation and the mapping onto middleware names.

ROS 2 names are not free-form strings; the rules (from the ROS 2 design) bound what is legal and
how a name reaches the middleware. A name is `/`-separated tokens of alphanumerics and
underscores; a token never starts with a digit; the name never has an empty token (`//`), a
doubled underscore (`__`), or a trailing `/`. A leading `/` makes it fully qualified; a leading
`~/` is the private namespace; balanced `{}` are runtime substitutions. On the wire DDS adds a
one-character subsystem prefix, `rt` for topics and `rq`/`rr` for the two halves of a service.

## enum `EntityKind`

The ROS 2 subsystem a name belongs to, which fixes its DDS prefix.

- `Topic` - A topic; DDS prefix `rt`.
- `ServiceRequest` - The request side of a service; DDS prefix `rq`.
- `ServiceResponse` - The reply side of a service; DDS prefix `rr`.

### `EntityKind::prefix`

Returns the DDS topic prefix for this subsystem.

**Returns**

`"rt"` for a topic, `"rq"` for a service request, `"rr"` for a service response.

```rust
fn prefix(self) -> &'static str
```

## fn `is_valid_name`

Returns whether a string is a valid ROS 2 topic or service name.

**Arguments**

* `name` - the candidate name.

**Returns**

`true` if `name` obeys the ROS 2 name rules: non-empty, no trailing `/`, no `//` or `__`, every
token is alphanumerics and underscores not starting with a digit, any `~` is the first character
and (if anything follows) is followed by `/`, and any `{}` substitutions are balanced and hold
only alphanumerics and underscores.

**Examples**

```
use pamoja_ros2::name::is_valid_name;

assert!(is_valid_name("/robot1/camera_left/image_raw"));
assert!(is_valid_name("~/setpoint"));
assert!(!is_valid_name("/2foo")); // a token may not start with a digit
assert!(!is_valid_name("/foo/")); // no trailing slash
assert!(!is_valid_name("/foo//bar")); // no empty token
```

```rust
fn is_valid_name(name: &str) -> bool
```

## fn `is_fully_qualified`

Returns whether a name is fully qualified: valid, absolute, and free of substitutions.

**Arguments**

* `name` - the candidate name.

**Returns**

`true` if `name` is valid, starts with `/`, and contains neither `~` nor `{}`. Only a fully
qualified name can be mapped onto the middleware, because the namespace is already resolved.

```rust
fn is_fully_qualified(name: &str) -> bool
```

## fn `dds_topic`

Maps a fully qualified ROS 2 name to its DDS topic name.

**Arguments**

* `fqn` - a fully qualified name (starting with `/`).
* `kind` - the subsystem, which selects the DDS prefix.

**Returns**

`Some(dds_name)` such as `rt/cmd_vel`, formed by prepending the subsystem prefix to the name;
`None` if `fqn` is not fully qualified.

**Examples**

```
use pamoja_ros2::name::{dds_topic, EntityKind};

assert_eq!(dds_topic("/foo", EntityKind::Topic).as_deref(), Some("rt/foo"));
assert_eq!(
    dds_topic("/robot1/camera_left/image_raw", EntityKind::Topic).as_deref(),
    Some("rt/robot1/camera_left/image_raw"),
);
assert_eq!(dds_topic("/add_two_ints", EntityKind::ServiceRequest).as_deref(), Some("rq/add_two_ints"));
assert_eq!(dds_topic("relative", EntityKind::Topic), None); // not fully qualified
```

```rust
fn dds_topic(fqn: &str, kind: EntityKind) -> Option <String>
```

## fn `percent_mangle`

Mangles a name by replacing each `/` with `%`, as `rmw_zenoh` does in liveliness tokens.

**Arguments**

* `name` - the name to mangle.

**Returns**

The name with every `/` replaced by `%`, for example `/chatter` becomes `%chatter`.

**Examples**

```
use pamoja_ros2::name::percent_mangle;

assert_eq!(percent_mangle("/chatter"), "%chatter");
assert_eq!(percent_mangle("/robot1/chatter"), "%robot1%chatter");
```

```rust
fn percent_mangle(name: &str) -> String
```

