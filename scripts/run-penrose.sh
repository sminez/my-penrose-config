#!/usr/bin/env bash

while true; do
  # log out to a file
  /home/innes/.config/penrose/target/release/penrose &> ~/.penrose.log
  mv ~/.penrose.log ~/prev-penrose.log
  export RESTARTED=true
done
