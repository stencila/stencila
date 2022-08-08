#!/usr/bin/env zsh

print "\U10ACDC" | tee /dev/stderr
while read -r task
do
  code=$(echo $task | sed 's/\xf4\x8b\x94\xa2/\n/g')
  printf -v lines "$code"
  eval "$lines"
  print "\U10ABBA" | tee /dev/stderr
done < "${1:-/dev/stdin}"
