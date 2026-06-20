//! Assembling the Zenoh key expression a `rmw_zenoh` peer subscribes to.
//!
//! `rmw_zenoh` puts every ROS 2 topic and service on a Zenoh key expression of the form
//! `<domain_id>/<fully_qualified_name>/<dds_type_name>/<type_hash>`, so a pamoja peer that builds
//! the same key talks to ROS 2 nodes over Zenoh with no DDS in the path. This assembles that key
//! from its parts and validates the result as a Zenoh key expression through [`pamoja_zenoh`], so a
//! malformed key is caught here rather than silently failing to match on the wire.

use crate::name::is_fully_qualified;
use crate::typehash::{dds_type_name, TypeHash};
use alloc::format;
use alloc::string::String;

/// Builds the `rmw_zenoh` key expression for a ROS 2 topic or service.
///
/// # Arguments
///
/// * `domain_id` - the ROS domain id (the `ROS_DOMAIN_ID`, default 0).
/// * `fqn` - the fully qualified name (starting with `/`), for example `/chatter`.
/// * `ros_type` - the interface type as `package/namespace/Type`, for example `std_msgs/msg/String`.
/// * `hash` - the message [`TypeHash`].
///
/// # Returns
///
/// `Some(key)` such as `0/chatter/std_msgs::msg::dds_::String_/RIHS01_...`; `None` if `fqn` is not
/// fully qualified, if `ros_type` is not a valid three-part interface type, or if the assembled key
/// is somehow not a valid Zenoh key expression.
///
/// # Examples
///
/// ```
/// use pamoja_ros2::key::entity_key;
/// use pamoja_ros2::typehash::TypeHash;
///
/// let hash =
///     TypeHash::parse("RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18")
///         .unwrap();
/// let key = entity_key(0, "/chatter", "std_msgs/msg/String", &hash).unwrap();
/// assert_eq!(
///     key,
///     "0/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18",
/// );
/// ```
pub fn entity_key(domain_id: u32, fqn: &str, ros_type: &str, hash: &TypeHash) -> Option<String> {
    if !is_fully_qualified(fqn) {
        return None;
    }
    let type_name = dds_type_name(ros_type)?;
    // `fqn` starts with `/`, so `{domain_id}{fqn}` joins as `0/chatter` with a single separator.
    let key = format!("{domain_id}{fqn}/{type_name}/{hash}");
    if pamoja_zenoh::keyexpr::is_valid(&key) {
        Some(key)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(text: &str) -> TypeHash {
        TypeHash::parse(text).unwrap()
    }

    #[test]
    fn matches_the_published_topic_key() {
        let h = hash("RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18");
        let key = entity_key(0, "/chatter", "std_msgs/msg/String", &h).unwrap();
        assert_eq!(
            key,
            "0/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18",
        );
    }

    #[test]
    fn matches_the_published_namespaced_key() {
        let h = hash("RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18");
        let key = entity_key(0, "/robot1/chatter", "std_msgs/msg/String", &h).unwrap();
        assert_eq!(
            key,
            "0/robot1/chatter/std_msgs::msg::dds_::String_/RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18",
        );
    }

    #[test]
    fn matches_the_published_service_key() {
        let h = hash("RIHS01_e118de6bf5eeb66a2491b5bda11202e7b68f198d6f67922cf30364858239c81a");
        let key = entity_key(2, "/add_two_ints", "example_interfaces/srv/AddTwoInts", &h).unwrap();
        assert_eq!(
            key,
            "2/add_two_ints/example_interfaces::srv::dds_::AddTwoInts_/RIHS01_e118de6bf5eeb66a2491b5bda11202e7b68f198d6f67922cf30364858239c81a",
        );
    }

    #[test]
    fn rejects_bad_inputs() {
        let h = hash("RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18");
        assert!(entity_key(0, "chatter", "std_msgs/msg/String", &h).is_none()); // not absolute
        assert!(entity_key(0, "/chatter", "std_msgs/String", &h).is_none()); // bad type
    }
}
