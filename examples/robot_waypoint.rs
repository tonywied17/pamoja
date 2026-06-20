//! Drive a rover safely, dead-reckon where it is, steer to a waypoint, and speak ROS 2.
//!
//! A small ground robot composed entirely from pamoja's robotics math, with no hardware. A body
//! command passes through the safety gate (speed and acceleration bounded, a deadman watchdog, an
//! e-stop), becomes differential wheel speeds, and feeds odometry that tracks the pose. Then a GPS
//! waypoint follower turns a position and heading into a steering command, and finally the same
//! motion is published the way a ROS 2 robot expects: a `geometry_msgs/Twist` as CDR on a
//! `rmw_zenoh` key. Composes `pamoja-kit` and `pamoja-ros2`.
//!
//! Run with: `cargo run -p pamoja-examples --example robot_waypoint`

use pamoja_kit::{
    obstacle_stop, Coordinate, DiffDrive, Limits, Odometry, SafetyGate, Twist, WaypointFollower,
};
use pamoja_ros2::key::entity_key;
use pamoja_ros2::msg::{Twist as TwistMsg, Vector3};
use pamoja_ros2::name::{dds_topic, EntityKind};
use pamoja_ros2::typehash::TypeHash;

fn main() {
    let dt = 0.1;

    // Scene 1: drive a differential rover through the safety gate and track its pose.
    println!("scene 1: bounded driving with odometry");
    let drive = DiffDrive::new(0.35); // wheels 0.35 m apart
    let limits = Limits::new(0.6, 1.5, 0.8, 3.0); // <=0.6 m/s, <=1.5 rad/s, eased
    let mut gate = SafetyGate::new(limits, 0.5); // stop if unfed for 0.5 s
    let mut odom = Odometry::at_origin();
    let desired = Twist::planar(0.6, 0.4); // forward, curving left

    for step in 1..=5 {
        gate.feed();
        let cmd = gate.command(desired, dt);
        let (left, right) = drive.wheel_speeds(cmd.vx, cmd.omega);
        let pose = odom.integrate(cmd.vx, cmd.omega, dt);
        println!(
            "  t={:.1}s  cmd=({:.2} m/s, {:.2} rad/s)  wheels=({:.2}, {:.2})  pose=({:.2}, {:.2}, {:.0} deg)",
            step as f32 * dt,
            cmd.vx,
            cmd.omega,
            left,
            right,
            pose.x,
            pose.y,
            pose.theta.to_degrees(),
        );
    }

    // The obstacle reflex cuts forward motion while leaving rotation free to turn away.
    let blocked = obstacle_stop(Twist::planar(0.6, 0.0), 0.25, 0.4);
    println!(
        "  obstacle 0.25 m ahead -> forward cut to {:.2} m/s",
        blocked.vx
    );

    // Losing the command stream trips the watchdog, and the gate commands a stop.
    let stalled = gate.command(desired, 1.0);
    println!(
        "  no command for 1.0 s -> watchdog stop: vx={:.2} m/s",
        stalled.vx
    );

    // Scene 2: steer toward a GPS waypoint from a position and a compass heading.
    println!("\nscene 2: steer toward a GPS waypoint");
    let follower = WaypointFollower::new(1.0, 2.0, 1.2, 1.0);
    let here = Coordinate::new(-1.2921, 36.8219);
    let target = Coordinate::new(-1.2900, 36.8219); // due north of here
    let guidance = follower.guide(here, 90.0, target); // currently facing east
    println!(
        "  distance {:.0} m, heading error {:.0} deg -> drive {:.2} m/s, turn {:.2} rad/s",
        guidance.distance_m, guidance.heading_error_deg, guidance.twist.vx, guidance.twist.omega,
    );

    // Scene 3: publish the motion the way a ROS 2 robot expects it.
    println!("\nscene 3: speak ROS 2 over Zenoh");
    let cmd_vel = TwistMsg {
        linear: Vector3::new(0.6, 0.0, 0.0),
        angular: Vector3::new(0.0, 0.0, 0.4),
    };
    println!("  cmd_vel Twist -> {} bytes of CDR", cmd_vel.to_cdr().len());
    println!(
        "  DDS topic name for /cmd_vel: {}",
        dds_topic("/cmd_vel", EntityKind::Topic).unwrap(),
    );

    // The rmw_zenoh key a Zenoh peer subscribes to, shown with the published /chatter reference.
    let hash =
        TypeHash::parse("RIHS01_df668c740482bbd48fb39d76a70dfd4bd59db1288021743503259e948f6b1a18")
            .unwrap();
    let key = entity_key(0, "/chatter", "std_msgs/msg/String", &hash).unwrap();
    println!("  rmw_zenoh key for /chatter: {key}");
}
