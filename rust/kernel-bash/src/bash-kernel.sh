#!/usr/bin/env bash

# To support versions of Bash below v4.2, we use
# the hexadecimal representations of flags. In Fish shell at least you
# can get the hexdecimal equivlents using `hexdump` e.g.
#
#   echo \U10ACDC | hexdump -C
#
# The "%" character must be escaped to "%%" so that `printf` does
# not interpret it as a format specification

printf "\xf4\x8a\xb3\x9c\n" | tee /dev/stderr
while read -r task
do
  code=$(echo $task | sed 's/\xf4\x8b\x94\xa2/\n/g')
  printf -v lines "${code//\%/%%}"
  eval "$lines"
  printf "\xf4\x8a\xae\xba\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
