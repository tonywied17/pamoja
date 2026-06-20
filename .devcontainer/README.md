# ROS 2 + Zenoh dev container

The host toolchain has no ROS 2, so the `pamoja-ros2` and `pamoja-zenoh` bridge is built and
tested inside the official `ros:jazzy` image with the Rust toolchain layered on top. Jazzy is the
pinned base: `r2r` builds against it with cargo alone, and `rmw_zenoh` is a released, Tier-1 RMW
there.

## Quick start

From the repo root, with Docker Desktop running:

```
cargo xtask ros
```

That builds the image from this directory and runs the pure-logic crate tests inside it, confirming
the toolchain and the ROS 2 + Zenoh middleware are in place. Extra arguments are appended to the
in-container `cargo test`, for example `cargo xtask ros --features bridge` once the live bridge
lands.

You can also open the folder in VS Code and "Reopen in Container" using `devcontainer.json`.

## What runs where

- The `no_std` pure-logic halves (`pamoja-zenoh` key expressions, `pamoja-ros2` names, type hashes,
  rmw_zenoh keys, and CDR) build and test anywhere, including normal CI, with no ROS 2.
- The live bridge (ROS 2 nodes, topics, services, and actions over `r2r`, and the Zenoh session)
  is std-only and runs here, against ROS 2 and Zenoh in the container.

`CARGO_TARGET_DIR` is redirected to `/tmp/target` so the container's Linux artifacts never collide
with the Windows build in the bind-mounted `target/`.
