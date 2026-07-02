# MAVLink SITL interop

Real-autopilot interop for `pamoja-mavlink`. The in-process
[`SitlAutopilot`](../crates/pamoja-mavlink/src/link.rs) tests run everywhere and prove the crate
against its own mock; the tests here prove it against an actual ArduPilot or PX4 flight stack, so
interop is real rather than self-referential.

Each autopilot builds into a self-contained Docker image that also carries the Rust toolchain, so
the interop test runs in the same container against a locally launched SITL, following the
`ros:jazzy` pattern the repo already uses for `pamoja-ros2`. Neither autopilot needs a GPU: both
run headless.

- `ardupilot.Dockerfile` builds `arducopter` SITL, which serves MAVLink over TCP on port 5760.
- `px4.Dockerfile` builds PX4 SITL with the headless jMAVSim simulator, which streams MAVLink over
  UDP to port 14550.
- `run-ardupilot.sh` / `run-px4.sh` launch the autopilot and run the ignored interop test
  ([`crates/pamoja-mavlink/tests/sitl.rs`](../crates/pamoja-mavlink/tests/sitl.rs)) with the
  endpoint set in `PAMOJA_SITL_TCP` or `PAMOJA_SITL_UDP`.

## Running

From the repo root, with Docker Desktop running:

```
cargo xtask sitl ardupilot   # build + run against ArduPilot SITL
cargo xtask sitl px4         # build + run against PX4 SITL
cargo xtask sitl all         # both
```

CI runs the same images and scripts in the `sitl` job (a matrix over the two autopilots). The
image build is the slow step; both autopilots are compiled from a pinned source release.

The interop test asserts protocol-level round-trips that hold for either autopilot: a heartbeat is
received, a mission uploads and downloads with the same item count, and a command is acknowledged.
It does not assert flight outcomes (a SITL without full sensor simulation may deny arming), only
that the exchanges complete correctly over the wire.
