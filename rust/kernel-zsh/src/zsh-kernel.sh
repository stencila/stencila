#!/usr/bin/env zsh

while read -r code
do
  print -v unescaped "$code"
  eval "$unescaped"
  print "\U10ACDC" | tee /dev/stderr
done < "${1:-/dev/stdin}"
