# PX4 SITL plus the Rust toolchain, for proving pamoja-mavlink against a real autopilot.
#
# PX4's software-in-the-loop runs the real flight stack driven by the jMAVSim simulator in
# headless mode (no GPU), which supplies the sensor data PX4 needs to boot fully and answer the
# mission and command protocols. The host toolchain (scoop) has no PX4, so this is how the
# real-autopilot interop is exercised. PX4 SITL streams MAVLink over UDP to port 14550.
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# PX4's setup script probes the distro with lsb_release and drives apt with sudo/gnupg, so those
# have to be present before it runs.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        git ca-certificates curl sudo lsb-release gnupg \
        python3 python3-pip python3-dev \
    && rm -rf /var/lib/apt/lists/*

# A pinned stable PX4 release. The main repo is cloned with full history (shallow clones break
# PX4's git-version header generation, which needs `git describe --tags`); only the submodules
# are shallow, which is where the bulk of the size is.
ARG PX4_TAG=v1.15.4
RUN git clone --branch ${PX4_TAG} --recurse-submodules --shallow-submodules \
        https://github.com/PX4/PX4-Autopilot.git /px4
WORKDIR /px4

# The PX4 SITL build prerequisites without the NuttX cross toolchain or the heavy Gazebo tools;
# jMAVSim only needs a JDK and ant, added directly.
RUN bash ./Tools/setup/ubuntu.sh --no-nuttx --no-sim-tools \
    && apt-get update \
    && apt-get install -y --no-install-recommends openjdk-17-jdk ant \
    && rm -rf /var/lib/apt/lists/*

# Compile the SITL flight-stack binary. px4_sitl_default is a pure build target: it compiles
# the binary and exits, unlike `make px4_sitl jmavsim`, which also launches the simulator. The
# jMAVSim jar is built on the first launch by the run script.
RUN make px4_sitl_default

# The Rust toolchain layered on top so the interop test runs in the same container.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal

# Keep the container's Linux build artifacts out of the bind-mounted host target/.
ENV CARGO_TARGET_DIR=/tmp/target

WORKDIR /work
