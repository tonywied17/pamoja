# ArduPilot SITL plus the Rust toolchain, for proving pamoja-mavlink against a real autopilot.
#
# ArduPilot's software-in-the-loop simulator runs the real flight-control MAVLink stack headless
# with no GPU, so the same image builds arducopter and runs the interop test against it. The host
# toolchain (scoop) has no ArduPilot, so this is how the real-autopilot interop is exercised. The
# arducopter SITL binary serves MAVLink over TCP on port 5760.
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# The prerequisites to build arducopter for the SITL board: a C/C++ toolchain, and the Python
# packages waf uses to generate the MAVLink message code (empy < 4 and pymavlink).
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        git ca-certificates curl \
        python3 python3-dev python3-pip python3-setuptools python3-wheel \
        build-essential ccache gawk make cmake pkg-config \
        libtool libxml2-dev libxslt1-dev \
    && rm -rf /var/lib/apt/lists/*

RUN python3 -m pip install --no-cache-dir \
        "empy==3.3.4" pymavlink future pexpect lxml

# A pinned stable Copter release, cloned shallow with its submodules to keep the image small.
ARG ARDUPILOT_TAG=Copter-4.5
RUN git clone --depth 1 --branch ${ARDUPILOT_TAG} \
        https://github.com/ArduPilot/ardupilot.git /ardupilot
WORKDIR /ardupilot
RUN git submodule update --init --recursive --depth 1

# Build the SITL copter binary (build/sitl/bin/arducopter).
RUN ./waf configure --board sitl \
    && ./waf copter

# The Rust toolchain layered on top so the interop test runs in the same container, mirroring the
# ROS 2 dev image.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal

# Keep the container's Linux build artifacts out of the bind-mounted host target/.
ENV CARGO_TARGET_DIR=/tmp/target

WORKDIR /work
