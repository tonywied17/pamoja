# pamoja-ros2::key

Generated from rustdoc by `cargo xtask docs` - do not edit by hand.

Assembling the Zenoh key expression a `rmw_zenoh` peer subscribes to.

`rmw_zenoh` puts every ROS 2 topic and service on a Zenoh key expression of the form
`<domain_id>/<fully_qualified_name>/<dds_type_name>/<type_hash>`, so a pamoja peer that builds
the same key talks to ROS 2 nodes over Zenoh with no DDS in the path. This assembles that key
from its parts and validates the result as a Zenoh key expression through [`pamoja_zenoh`], so a
malformed key is caught here rather than silently failing to match on the wire.

## fn `entity_key`

Builds the `rmw_zenoh` key expression for a ROS 2 topic or service.

**Arguments**

* `domain_id` - the ROS domain id (the `ROS_DOMAIN_ID`, default 0).
* `fqn` - the fully qualified name (starting with `/`), for example `/chatter`.
* `ros_type` - the interface type as `package/namespace/Type`, for example `std_msgs/msg/String`.
* `hash` - the message [`TypeHash`].

**Returns**

`Some(key)` such as `0/chatter/std_msgs::msg::dds_::String_/RIHS01_...`; `None` if `fqn` is not
fully qualified, if `ros_type` is not a valid three-part interface type, or if the assembled key
is somehow not a valid Zenoh key expression.

**Examples**

```
use pamoja_ros2::key::entity_key;
use pamoja_ros2::typehash::TypeHash;

let hash =
    TypeHash::parse("RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18")
        .unwrap();
let key = entity_key(0, "/chatter", "std_msgs/msg/String", &hash).unwrap();
assert_eq!(
    key,
    "0/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18",
);
```

```rust
fn entity_key(domain_id: u32, fqn: &str, ros_type: &str, hash: &TypeHash) -> Option <String>
```

