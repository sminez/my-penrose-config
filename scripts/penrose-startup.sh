#! /usr/bin/env bash
# ----------------------------------------
# Bootstrap the start of a penrose session
# >> This get's run on restart as well!
# ----------------------------------------

# Make sure we only run once
pid=$$
pgrep -fi penrose-startup.sh | grep -v "^$pid$" | xargs kill
pgrep -fi penrose-stat.zsh | xargs kill

# Set screen resolutions (add additional screens here)
xrandr --output HDMI-2 --auto --right-of eDP-1 &

# fix a couple of quirks with my thinkpad: enable tap-click for the touchpad
# and slow down the track point accelleration
xinput --set-prop "11" "libinput Tapping Enabled" 1
xinput --set-prop "12" "libinput Accel Speed" 0.0

# Keyboard overrides
setxkbmap -option caps:ctrl_modifier

xsetroot -cursor_name left_ptr

running() { pgrep -fi "$1" >/dev/null; }

# running kdeconnnectd || /usr/lib/kdeconnectd &
# running picom || picom &
running nm-applet || nm-applet &
running udiskie || udiskie -a -n -t &
running xautolock || xautolock \
  -detectsleep \
  -time 3 \
  -locker "/usr/local/bin/lock-screen" \
  -notify 30 \
  -notifier "notify-send -u critical -t 120 -- 'LOCKING screen in 30 seconds...'" &
running volumeicon || volumeicon &
running dunst || dunst &
running blueman-applet || blueman-applet &
running xfce4-power-manager || xfce4-power-manager &
running gnome-keyring-daemon || gnome-keyring-daemon --start --components=pkcs11,secrets,ssh &

"$HOME/.fehbg"
/usr/local/scripts/penrose-stat.zsh &

# see /usr/local/bin/run-penrose
[[ -z "$RESTARTED" ]] && /usr/local/scripts/unlock-ssh.sh &
