#!/usr/bin/env bash

while read -r code
do
  printf -v unescaped "$code"
  eval "$unescaped"
  # To support versions of Bash v4.2 and below, we use
  # the hexadecimal representation of \U10ACDC
  echo -e "\xf4\x8a\xb3\x9c" | tee /dev/stderr
done < "${1:-/dev/stdin}"
