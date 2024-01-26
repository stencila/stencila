#!/usr/bin/env bash

READY=$(printf "\U10ACDC")
LINE=$(printf "\U10ABBA")
EXEC=$(printf "\U10B522")
EVAL=$(printf "\U1010CC")
FORK=$(printf "\U10DE70")


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
  
  if echo "$lines" | grep -q "^$EXEC"; then
    # Execute code
    eval $(echo "$lines" | sed "s/^$EXEC//")
  elif echo "$lines" | grep -q "^$EVAL"; then
    # Evaluate code (integer expressions only)
    eval $(echo "$lines" | sed "s/^$EVAL/echo \$\(\(/; s/$/\)\)/")
  fi
  
  # Print READY flag on stdout and stderr
  printf "$READY\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
