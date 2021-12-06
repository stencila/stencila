#!/usr/bin/env bash

while read -r code
do
  printf -v unescaped "$code"
  eval "$unescaped"
  # To support versions of Bash below v4.2, we use
  # the hexadecimal representation of \U10ACDC
  printf "\xf4\x8a\xb3\x9c\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
