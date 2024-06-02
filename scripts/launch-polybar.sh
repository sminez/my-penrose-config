#!/usr/bin/env bash

killall -q polybar
while pgrep -u "$UID" -x polybar >/dev/null; do
  sleep 1;
done

for m in $(xrandr --query | grep -e "\bconnected" | cut -d" " -f1); do
  MONITOR=$m polybar --reload "$m" &
done

# polybar main -c ~/.config/polybar/config.ini &
