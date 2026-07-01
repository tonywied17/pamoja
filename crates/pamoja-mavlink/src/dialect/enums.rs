//! A useful subset of the common-dialect enumerations, as named constants.
//!
//! MAVLink enum fields ride on the wire as plain integers, so the typed messages store
//! them as integers and these constants give the common values readable names. They are
//! grouped by enum; only the values a ground station and an autopilot reach for most often
//! are included, and any other value a field carries is still valid.

#![allow(missing_docs)]

/// `MAV_TYPE`: the kind of vehicle or component a [`Heartbeat`](super::Heartbeat) describes.
pub mod mav_type {
    pub const GENERIC: u8 = 0;
    pub const FIXED_WING: u8 = 1;
    pub const QUADROTOR: u8 = 2;
    pub const COAXIAL: u8 = 3;
    pub const HELICOPTER: u8 = 4;
    pub const ANTENNA_TRACKER: u8 = 5;
    pub const GCS: u8 = 6;
    pub const HEXAROTOR: u8 = 13;
    pub const OCTOROTOR: u8 = 14;
    pub const SUBMARINE: u8 = 12;
    pub const GROUND_ROVER: u8 = 10;
    pub const SURFACE_BOAT: u8 = 11;
    pub const ONBOARD_CONTROLLER: u8 = 18;
}

/// `MAV_AUTOPILOT`: which autopilot stack a [`Heartbeat`](super::Heartbeat) comes from.
pub mod mav_autopilot {
    pub const GENERIC: u8 = 0;
    pub const ARDUPILOTMEGA: u8 = 3;
    pub const PX4: u8 = 12;
    pub const INVALID: u8 = 8;
}

/// `MAV_STATE`: the system status carried by a [`Heartbeat`](super::Heartbeat).
pub mod mav_state {
    pub const UNINIT: u8 = 0;
    pub const BOOT: u8 = 1;
    pub const CALIBRATING: u8 = 2;
    pub const STANDBY: u8 = 3;
    pub const ACTIVE: u8 = 4;
    pub const CRITICAL: u8 = 5;
    pub const EMERGENCY: u8 = 6;
    pub const POWEROFF: u8 = 7;
    pub const FLIGHT_TERMINATION: u8 = 8;
}

/// `MAV_MODE_FLAG`: bits of the base mode field of a [`Heartbeat`](super::Heartbeat).
pub mod mav_mode_flag {
    pub const CUSTOM_MODE_ENABLED: u8 = 1;
    pub const TEST_ENABLED: u8 = 2;
    pub const AUTO_ENABLED: u8 = 4;
    pub const GUIDED_ENABLED: u8 = 8;
    pub const STABILIZE_ENABLED: u8 = 16;
    pub const HIL_ENABLED: u8 = 32;
    pub const MANUAL_INPUT_ENABLED: u8 = 64;
    pub const SAFETY_ARMED: u8 = 128;
}

/// `MAV_CMD`: command ids for [`CommandLong`](super::CommandLong) and [`CommandInt`](super::CommandInt).
pub mod mav_cmd {
    pub const NAV_WAYPOINT: u16 = 16;
    pub const NAV_LOITER_UNLIM: u16 = 17;
    pub const NAV_RETURN_TO_LAUNCH: u16 = 20;
    pub const NAV_LAND: u16 = 21;
    pub const NAV_TAKEOFF: u16 = 22;
    pub const DO_SET_MODE: u16 = 176;
    pub const DO_SET_HOME: u16 = 179;
    pub const DO_SET_SERVO: u16 = 183;
    pub const COMPONENT_ARM_DISARM: u16 = 400;
    pub const REQUEST_MESSAGE: u16 = 512;
}

/// `MAV_RESULT`: the outcome a [`CommandAck`](super::CommandAck) reports.
pub mod mav_result {
    pub const ACCEPTED: u8 = 0;
    pub const TEMPORARILY_REJECTED: u8 = 1;
    pub const DENIED: u8 = 2;
    pub const UNSUPPORTED: u8 = 3;
    pub const FAILED: u8 = 4;
    pub const IN_PROGRESS: u8 = 5;
    pub const CANCELLED: u8 = 6;
}

/// `MAV_MISSION_RESULT`: the outcome a [`MissionAck`](super::MissionAck) reports.
pub mod mav_mission_result {
    pub const ACCEPTED: u8 = 0;
    pub const ERROR: u8 = 1;
    pub const UNSUPPORTED: u8 = 3;
    pub const INVALID: u8 = 5;
    pub const INVALID_SEQUENCE: u8 = 12;
    pub const CANCELLED: u8 = 13;
}

/// `MAV_FRAME`: the coordinate frame of a mission item or position target.
pub mod mav_frame {
    pub const GLOBAL: u8 = 0;
    pub const LOCAL_NED: u8 = 1;
    pub const MISSION: u8 = 2;
    pub const GLOBAL_RELATIVE_ALT: u8 = 3;
    pub const GLOBAL_INT: u8 = 5;
    pub const GLOBAL_RELATIVE_ALT_INT: u8 = 6;
}
