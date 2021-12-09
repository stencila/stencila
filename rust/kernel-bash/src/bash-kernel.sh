#!/usr/bin/env bash

# To support versions of Bash below v4.2, we use
# the hexadecimal representations of flags

printf "\xf4\x8a\xb3\x9c\n" | tee /dev/stderr
while read -r code
do
  printf -v unescaped "$code"
  eval "$unescaped"
  printf "\xf4\x8a\xae\xba\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
