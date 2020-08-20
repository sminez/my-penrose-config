#! /usr/bin/env bash
# ----------------------------------------
# Bootstrap the start of a penrose session
# >> This get's run on restart as well!
# ----------------------------------------

# Make sure we only run once
pid=$$
pgrep -fi penrose-startup.sh | grep -v "^$pid$" | xargs kill

# Set screen resolutions (add additional screens here)
xrandr --output HDMI-1 --auto --right-of eDP-1 &

running() { pgrep -fi "$1" >/dev/null; }

running kdeconnnectd || /usr/lib/kdeconnectd &
running nm-applet || nm-applet &
running udiskie || udiskie -a -n -t &
running xautolock || xautolock \
  -detectsleep \
  -time 3 \
  -locker "$HOME/bin/lock-screen" \
  -notify 30 \
  -notifier "notify-send -u critical -t 120 -- 'LOCKING screen in 30 seconds...'" &
running volumeicon || volumeicon &
running dunst || dunst &
running blueman-applet || blueman-applet &
running xfce4-power-manager || xfce4-power-manager &
running gnome-keyring-daemon || gnome-keyring-daemon --start --components=pkcs11,secrets,ssh &

"$HOME/.fehbg"
"$HOME/.config/polybar/launch.sh" &

# see run-penrose.sh
[[ -z "$RESTARTED" ]] && "$HOME/bin/unlock-ssh.sh" &
