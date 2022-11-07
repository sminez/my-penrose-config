#! /usr/bin/env bash
# ----------------------------------------
# Bootstrap the start of a penrose session
# >> This get's run on restart as well!
# ----------------------------------------

# Make sure we only run once
pid=$$
pgrep -fi penrose-startup.sh | grep -v "^$pid$" | xargs -I{} kill {}

# Set screen resolutions (add additional screens here)
xrandr --output HDMI-2 --auto --right-of eDP-1 &

# fix a couple of quirks with my thinkpad: enable tap-click for the touchpad
# and slow down the track point accelleration
xinput --set-prop "11" "libinput Tapping Enabled" 1
xinput --set-prop "12" "libinput Accel Speed" 0.0

# Keyboard overrides
setxkbmap -option caps:ctrl_modifier
xsetroot -cursor_name left_ptr

# pkill -fi stalonetray; stalonetray -bg '#282828' --icon-size 18 &
pkill -fi picom; picom &
pkill -fi nm-applet; nm-applet &
pkill -fi udiskie; udiskie -a -n -t &
# pkill -fi xautolock; xautolock \
#   -detectsleep \
#   -time 3 \
#   -locker "/usr/local/bin/lock-screen" \
#   -notify 30 \
#   -notifier "notify-send -u critical -t 120 -- 'LOCKING screen in 30 seconds...'" &
pkill -fi volumeicon; volumeicon &
pkill -fi dunst; dunst &
pkill -fi blueman-applet; blueman-applet &
pkill -fi xfce4-power-man; xfce4-power-manager &  # for some reason, this ends up running as xcfe4-power-man
pkill -fi gnome-keyring-daemon; gnome-keyring-daemon --start --components=pkcs11,secrets,ssh &

"$HOME/.fehbg"

# see /usr/local/bin/run-penrose
[[ -z "$RESTARTED" ]]; /usr/local/scripts/unlock-ssh.sh &
