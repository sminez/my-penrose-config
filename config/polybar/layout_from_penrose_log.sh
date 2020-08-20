#!/usr/bin/env sh
tail -F ~/.penrose.log |
    awk '/\[INFO\] ACTIVE_LAYOUT/ { print $4 }'
