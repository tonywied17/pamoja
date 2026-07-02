#!/usr/bin/env bash
# Launch ArduPilot SITL and run the pamoja-mavlink interop test against it.
#
# Run inside the pamoja-sitl-ardupilot image with the repo mounted at /work. Starts the
# arducopter SITL binary (which serves MAVLink over TCP on port 5760), waits for it to boot,
# then runs the ignored interop test pointed at that endpoint.
set -uo pipefail

cd /ardupilot

# sim_vehicle.py is ArduPilot's canonical SITL launcher: it sets up the working directory,
# storage (EEPROM), and parameters the raw binary does not, then serves MAVLink for a ground
# station on TCP 5760. --no-mavproxy keeps it headless; --no-rebuild uses the binary already
# built into the image; -w wipes to a clean state.
python3 Tools/autotest/sim_vehicle.py -v ArduCopter -f quad \
    --no-mavproxy --no-rebuild -w \
    >/tmp/arducopter.log 2>&1 &
SITL_PID=$!

cleanup() {
    kill "${SITL_PID}" 2>/dev/null || true
    pkill -f arducopter 2>/dev/null || true
    pkill -f sim_vehicle 2>/dev/null || true
}
trap cleanup EXIT

echo "waiting for ArduPilot SITL to boot..."
sleep 45
if ! pgrep -f arducopter >/dev/null 2>&1; then
    echo "ArduPilot SITL is not running; log:"
    tail -n 40 /tmp/arducopter.log
    exit 1
fi

cd /work
export PAMOJA_SITL_TCP=127.0.0.1:5760
cargo test -p pamoja-mavlink --test sitl -- --ignored --nocapture
STATUS=$?
echo "===== arducopter log tail ====="
tail -n 40 /tmp/arducopter.log || true
exit $STATUS
