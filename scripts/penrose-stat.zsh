#! /usr/bin/env zsh
# A super simple status bar for dwm

BATTERY_INSTANCE="BAT1"
NET_INTERFACE="$(ip route | awk '/^default/ { print $5 ; exit }')"

function get_ip {
  local state="$(cat /sys/class/net/"$NET_INTERFACE"/operstate)"
  [[ "$state" = 'down' ]] && echo -n  "ip: ❌" || echo -n "$(ip route get 1 | cut -d' ' -f3)"
}

function get_battery {
  local state perc icon

  upower -i "/org/freedesktop/UPower/devices/battery_$BATTERY_INSTANCE" |
    awk '/state|percentage/ { print $2 }' |
    xargs |
    tr -d '%' |
    read state perc

  if [[ "$state" = "charging" ]] || [[ "$state" = "fully-charged" ]]; then
    icon=""
  elif (( perc < 10 )); then
    icon=""
  elif (( perc == 100 )); then
    icon=""
  else
    case "$(echo "$perc" | cut -c1)" in
        9) icon="";;
      7|8) icon="";;
      5|6) icon="";;
      3|4) icon="";;
      1|2) icon="";;
    esac
  fi

  echo -n "$icon $perc%"
}

function get_volume {
  local vol="$(amixer get Master | tail -n1 | sed -r 's/.*\[(.*)%\].*/\1/')"
  (( vol > 0 )) && echo -n " $vol%" || echo -n " ❌"
}

function get_layout {
  echo -n "[$(setxkbmap -verbose 10 | awk -F':' '/layout/ { print $2 }' | xargs)]"
}

# == main loop ==
while true; do
  xsetroot -name "<$(iwgetid -r)> $(get_battery) $(get_volume) $(get_layout) $(date '+%c')"
  sleep 2
done
