//! Message type identity: the DDS type name and the RIHS01 type hash.
//!
//! Two peers only exchange a message if they agree on its type. ROS 2 pins that agreement two ways:
//! a DDS type name derived from the interface (`std_msgs/msg/String` becomes
//! `std_msgs::msg::dds_::String_`), and a structural type hash (REP-2011's RIHS01, a SHA-256 over
//! the type's description, written `RIHS01_` followed by 64 hex digits). This module derives the
//! type name and parses and formats the hash. Computing the hash from a type description is part of
//! the live bridge, where it is checked against the value `rosidl` emits.

use alloc::format;
use alloc::string::String;
use core::fmt;

/// The length in bytes of a RIHS01 hash (a SHA-256 digest).
const HASH_LEN: usize = 32;

/// A parsed ROS 2 type hash in the RIHS01 scheme (REP-2011).
///
/// The string form is `RIHS01_` followed by 64 lowercase hex digits, the version prefix plus a
/// SHA-256 digest of the type's description.
///
/// # Examples
///
/// ```
/// use pamoja_ros2::typehash::TypeHash;
///
/// // The published hash of std_msgs/msg/String round-trips through parse and display.
/// let text = "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18";
/// let hash = TypeHash::parse(text).unwrap();
/// assert_eq!(hash.to_string(), text);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeHash {
    digest: [u8; HASH_LEN],
}

impl TypeHash {
    /// Parses a RIHS01 hash string.
    ///
    /// # Arguments
    ///
    /// * `text` - the candidate hash, expected as `RIHS01_` plus 64 lowercase hex digits.
    ///
    /// # Returns
    ///
    /// `Some(hash)` if `text` is a well-formed RIHS01 string, otherwise `None`.
    pub fn parse(text: &str) -> Option<Self> {
        let hex = text.strip_prefix("RIHS01_")?;
        if hex.len() != HASH_LEN * 2 {
            return None;
        }
        let bytes = hex.as_bytes();
        let mut digest = [0u8; HASH_LEN];
        let mut i = 0;
        while i < HASH_LEN {
            let hi = hex_value(bytes[i * 2])?;
            let lo = hex_value(bytes[i * 2 + 1])?;
            digest[i] = (hi << 4) | lo;
            i += 1;
        }
        Some(Self { digest })
    }

    /// Returns the raw 32-byte digest.
    ///
    /// # Returns
    ///
    /// The SHA-256 digest carried by the hash.
    pub fn digest(&self) -> [u8; HASH_LEN] {
        self.digest
    }
}

impl fmt::Display for TypeHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RIHS01_")?;
        for byte in self.digest {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

// Returns the value of a single lowercase-or-digit hex character, or `None` if it is not hex.
fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

/// Derives the DDS type name from a ROS 2 interface type.
///
/// # Arguments
///
/// * `ros_type` - the interface type as `package/namespace/Type`, for example `std_msgs/msg/String`.
///
/// # Returns
///
/// `Some(dds_name)` such as `std_msgs::msg::dds_::String_`, joining the parts with `::`, inserting
/// the `dds_` namespace, and suffixing the type with `_`; `None` if `ros_type` is not three
/// non-empty `/`-separated parts.
///
/// # Examples
///
/// ```
/// use pamoja_ros2::typehash::dds_type_name;
///
/// assert_eq!(dds_type_name("std_msgs/msg/String").as_deref(), Some("std_msgs::msg::dds_::String_"));
/// assert_eq!(
///     dds_type_name("example_interfaces/srv/AddTwoInts").as_deref(),
///     Some("example_interfaces::srv::dds_::AddTwoInts_"),
/// );
/// assert_eq!(dds_type_name("std_msgs/String"), None); // missing the namespace part
/// ```
pub fn dds_type_name(ros_type: &str) -> Option<String> {
    let mut parts = ros_type.split('/');
    let package = parts.next().filter(|p| !p.is_empty())?;
    let namespace = parts.next().filter(|p| !p.is_empty())?;
    let type_name = parts.next().filter(|p| !p.is_empty())?;
    if parts.next().is_some() {
        return None;
    }
    Some(format!("{package}::{namespace}::dds_::{type_name}_"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_the_published_hash() {
        let text = "RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18";
        let hash = TypeHash::parse(text).unwrap();
        assert_eq!(hash.to_string(), text);
        assert_eq!(hash.digest()[0], 0xdf);
        assert_eq!(hash.digest()[31], 0x18);
    }

    #[test]
    fn rejects_malformed_hashes() {
        assert!(TypeHash::parse("RIHS01_tooshort").is_none());
        assert!(TypeHash::parse("df668c74").is_none()); // missing prefix
        assert!(TypeHash::parse(
            "RIHS02_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18"
        )
        .is_none());
        // Uppercase hex is not the canonical lowercase form.
        assert!(TypeHash::parse(
            "RIHS01_DF668C740482BBD48FB39D76A70DFD4BD59DB1288021743503259E948F6B1A18"
        )
        .is_none());
    }

    #[test]
    fn derives_dds_type_names() {
        assert_eq!(
            dds_type_name("std_msgs/msg/String").as_deref(),
            Some("std_msgs::msg::dds_::String_"),
        );
        assert_eq!(
            dds_type_name("geometry_msgs/msg/Twist").as_deref(),
            Some("geometry_msgs::msg::dds_::Twist_"),
        );
        assert_eq!(
            dds_type_name("example_interfaces/srv/AddTwoInts").as_deref(),
            Some("example_interfaces::srv::dds_::AddTwoInts_"),
        );
        assert_eq!(dds_type_name("std_msgs/String"), None);
        assert_eq!(dds_type_name("a/b/c/d"), None);
    }
}
