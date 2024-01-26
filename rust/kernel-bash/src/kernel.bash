#!/usr/bin/env bash

READY=$(printf "\U10ACDC")
LINE=$(printf "\U10ABBA")
EXEC=$(printf "\U10B522")
EVAL=$(printf "\U1010CC")
FORK=$(printf "\U10DE70")
LIST=$(printf "\U10C155")
GET=$(printf "\U10A51A")
SET=$(printf "\U107070")
REMOVE=$(printf "\U10C41C")
END=$(printf "\U10CB40")

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
  elif echo "$lines" | grep -q "^$LIST"; then
    # Return a list of variables
    printenv | awk -v end="$END" -F= '{print "{\"type\":\"Variable\",\"kind\":\"String\",\"name\":\"" $1 "\"}" end}'
  elif echo "$lines" | grep -q "^$GET"; then
    # Get a variable (without trailing newline)
    printf '%s' $(printenv $(echo "$lines" | sed "s/^$GET//"))
  elif echo "$lines" | grep -q "^$SET"; then
    # Set a variable
    eval $(echo "$lines" | sed "s/^$SET//" | awk 'NR==1{var=$0} NR==2{print "export " var "=\"" $0 "\""}')
  elif echo "$lines" | grep -q "^$REMOVE"; then
    # Remove a variable
    unset $(echo "$lines" | sed "s/^$REMOVE//")
  fi
  
  # Print READY flag on stdout and stderr
  printf "$READY\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
