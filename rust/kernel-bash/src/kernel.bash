#!/usr/bin/env bash

READY=$(printf "\U10ACDC")
EXEC=$(printf "\U10B522")
FORK=$(printf "\U10DE70")
LINE=$(printf "\U10ABBA")

# Print initial READY flag on stdout and stderr
printf "$READY\n" | tee /dev/stderr

# Read each stdin line as a task
while read -r line
do
  # Unescape newlines in code
  unescaped=$(echo "$line" | sed "s/$LINE/\n/g")
  # Use printf to expand newlines (escape % in code to avoid being
  # interpreted by printf as format spec)
  printf -v lines "${unescaped//\%/%%}"
  # Execute the code (only EXEC tasks supported at present)
  if echo "$lines" | grep -q "^$EXEC"; then
    code=$(echo "$lines" | sed "s/^$EXEC//")
    eval $code
  fi
  # Print READY flag on stdout and stderr
  printf "$READY\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
