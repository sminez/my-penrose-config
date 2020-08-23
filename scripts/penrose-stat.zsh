#! /usr/bin/env zsh
# A super simple status bar for penrose

AUDIO_CHANNEL="Master"
BATTERIES=("BAT1" "BAT0")
INTERFACE="$(iwgetid | cut -d' ' -f1)"
POLL=2

function get_wifi {
  # local quality=$(grep $INTERFACE /proc/net/wireless | awk '{ print int($3 * 100 / 70) }')
  # (( quality >= 81 )) && icon="● "
  # (( quality <= 80 )) && icon="◑ "
  # (( quality <= 60 )) && icon="◑ "
  # (( quality <= 40 )) && icon="◔ "
  # (( quality <= 20 )) && icon="○ "
  echo -n "<$(iwgetid -r)>"
}

function get_battery {
  local state perc icon

  for battery in $BATTERIES; do
    upower -i "/org/freedesktop/UPower/devices/battery_$battery" |
      awk '/state|percentage/ { print $2 }' |
      xargs |
      tr -d '%' |
      read state perc

    if [[ "$state" = "charging" ]]; then
      icon=""
    elif (( perc < 10 )); then
      icon=""
    elif (( perc == 100 )) || [[ "$state" = "fully-charged" ]]; then
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

    echo -n "$icon $perc% "
  done | xargs
}

function get_volume {
  local vol="$(amixer get $AUDIO_CHANNEL | tail -n1 | sed -r 's/.*\[(.*)%\].*/\1/')"
  (( vol > 0 )) && echo -n " $vol%" || echo -n " ❌"
}

function get_keyboard_layout {
  echo -n "[$(setxkbmap -verbose 10 | awk -F':' '/layout/ { print $2 }' | xargs)]"
}

function get_status {
  echo -n "$(get_wifi) $(get_battery) $(get_volume) $(get_keyboard_layout) $(date '+%F %R')"
}

# == main loop ==
current_status=""

while true; do
  new_status="$(get_status)"
  [ "$current_status" = "$new_status" ] || echo "$new_status"
  current_status="$new_status"
  sleep $POLL
done
