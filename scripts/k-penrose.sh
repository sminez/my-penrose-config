#! /usr/bin/env zsh
# Wrapper around k to allow penrose to open URLs from a transient window

BROWSER=qutebrowser

float_class="$1"
outfile="$(mktemp /tmp/k-urls-XXX)"

alacritty \
  --class "$float_class" \
  --command zsh -c "k --no-color | grep -E \"^http\" > $outfile" &

lines=$(echo $outfile | entr -panz cat "$outfile")
rm "$outfile"

if (( $(echo $lines | wc -l) > 1 )); then
  url=$(echo $lines | dmenu -p 'select which url to open: ')
else
  url=$lines
fi

if [[ -n "$url" ]]; then
  "$BROWSER" "$url"
  wmctrl -a "$BROWSER"
fi
