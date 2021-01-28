#! /usr/bin/env bash
# Unlock my RSA key. Called on startup
sleep 1
output=$(echo | SSH_ASKPASS=/usr/local/scripts/unlock-ssh-helper.sh ssh-add 2>&1)
notify-send "SSH Unlock Output" "$output"
