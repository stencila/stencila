#!/usr/bin/env zsh

print "\U10ACDC" | tee /dev/stderr
while read -r task
do
  print -v unescaped "$task"
  eval "$unescaped"
  print "\U10ABBA" | tee /dev/stderr
done < "${1:-/dev/stdin}"
