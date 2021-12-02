#!/usr/bin/env bash

while read -r code
do
  printf -v unescaped "$code"
  eval "$unescaped"
  echo -e "\U10ACDC" | tee /dev/stderr
done < "${1:-/dev/stdin}"
