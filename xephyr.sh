#!/usr/bin/env bash
# Run the window manager in an embeded Xephyr session.
#
# usage:
#   APP=st ./xephyr.sh
   
CUR_DIR="$(dirname "$(readlink -f "$0")")"
SCREEN_SIZE=${SCREEN_SIZE:-1200x900}
XDISPLAY=${XDISPLAY:-:2}

BIN_NAME=${BIN_NAME:-penrose}
APP=${APP:-st}

cargo build

rm -f "$CUR_DIR/xephyr.log"
touch "$CUR_DIR/xephyr.log"

Xephyr +extension RANDR -screen "$SCREEN_SIZE" "$XDISPLAY" -ac &
XEPHYR_PID=$!

sleep 1
env DISPLAY="$XDISPLAY" "$CUR_DIR/target/debug/$BIN_NAME" 2>&1 "$CUR_DIR/xephyr.log" &
WM_PID=$!

trap "kill $XEPHYR_PID && kill $WM_PID" SIGINT SIGTERM exit

env DISPLAY="$XDISPLAY" "$APP" &

tail -f "$CUR_DIR/xephyr.log"

wait $WM_PID
kill $XEPHYR_PID
