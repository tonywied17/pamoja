#!/usr/bin/env bash
# Launch PX4 SITL (jMAVSim, headless) and run the pamoja-mavlink interop test against it.
#
# Run inside the pamoja-sitl-px4 image with the repo mounted at /work. Starts PX4 SITL with the
# headless jMAVSim simulator (which supplies the sensor data PX4 needs to boot), waits for it to
# come up, then runs the ignored interop test against PX4's UDP MAVLink stream on port 14550.
set -uo pipefail

cd /px4

# jMAVSim headless; </dev/null keeps the px4 console from blocking on stdin.
HEADLESS=1 make px4_sitl jmavsim </dev/null >/tmp/px4.log 2>&1 &
PX4_PID=$!

# Kill the SITL process tree on exit, matching patterns specific enough not to match this
# script itself (its path contains "px4"), so cleanup never SIGTERMs the run.
cleanup() {
    kill "${PX4_PID}" 2>/dev/null || true
    pkill -f 'px4_sitl_default/bin/px4' 2>/dev/null || true
    pkill -f 'jmavsim_run.jar' 2>/dev/null || true
}
trap cleanup EXIT

echo "waiting for PX4 SITL to boot (jMAVSim headless)..."
sleep 75
if ! kill -0 "${PX4_PID}" 2>/dev/null; then
    echo "PX4 SITL exited before it was ready; log:"
    tail -n 60 /tmp/px4.log
    exit 1
fi

cd /work
export PAMOJA_SITL_UDP=0.0.0.0:14550
cargo test -p pamoja-mavlink --test sitl -- --ignored --nocapture
STATUS=$?
exit $STATUS
