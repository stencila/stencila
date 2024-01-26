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
  # Split the line into lines using the LINE flag as separator
  IFS="$LINE" read -ra lines <<< "$line"
  
  # Switch on the task flag (the first line)
  case "${lines[0]}" in 
    "$EXEC")
      # Execute remaining lines
      eval $(printf "%s\n" "${lines[@]:1}")
      ;;
    "$EVAL")
      # Evaluate second line (integer expressions only)
      eval "echo $((${lines[1]}))"
      ;;
    "$LIST")
      # Return a list of variables (with END flag after each)
      printenv | awk -v end="$END" -F= '{print "{\"type\":\"Variable\",\"kind\":\"String\",\"name\":\"" $1 "\"}" end}'
      ;;
    "$GET")
      # Get a variable (using printf to avoid trailing newline)
      printf '%s' $(printenv ${lines[1]})
      ;;
    "$SET")
      # Set a variable
      eval "export ${lines[1]}=${lines[2]}"
      ;;
    "$REMOVE")
      # Remove a variable
      unset "${lines[1]}"
      ;;
    *)
      echo "Unrecognised flag" >&2
      ;;
  esac
  
  # Print READY flag on stdout and stderr
  printf "$READY\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
