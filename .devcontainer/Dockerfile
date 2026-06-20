# Reproducible build/test environment for the ROS 2 + Zenoh bridge.
#
# ROS 2 Jazzy is the pinned base: r2r builds against it with nothing but cargo, and rmw_zenoh
# is a released, Tier-1 RMW there, so the same image covers the DDS and the Zenoh paths. The
# host toolchain (scoop) has no ROS 2, so the bridge is always built and tested in here.
FROM ros:jazzy

# Zenoh as a ROS 2 RMW (the edge-to-edge / fleet transport and DDS alternative), plus the
# system libraries r2r needs to bind the ROS 2 client libraries.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ros-jazzy-rmw-zenoh-cpp \
        ros-jazzy-example-interfaces \
        build-essential \
        pkg-config \
        libclang-dev \
        clang \
        curl \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# The Rust toolchain layered on top. r2r and zenoh want a current stable, newer than the
# workspace MSRV; the live bridge is std-only and built only in this container, so a higher
# bridge toolchain does not affect the no_std core that must stay on the pinned MSRV.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal \
    && rustup component add rustfmt clippy

# Keep the container's Linux build artifacts out of the bind-mounted host target/, which holds
# the Windows build, so the two never collide.
ENV CARGO_TARGET_DIR=/tmp/target

WORKDIR /work
