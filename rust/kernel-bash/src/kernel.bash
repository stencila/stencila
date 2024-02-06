#!/usr/bin/env bash

# During development it can be useful to run this kernel script directly e.g.
#
#     DEV=true bash rust/kernel-bash/src/kernel.bash
#
# Use Ctrl+D to quit, since Ctrl+C is trapped for interrupting kernel tasks.

if [ "$DEV" == "true" ]; then
  READY="READY"
  LINE="|"
  EXEC="EXEC"
  EVAL="EVAL"
  FORK="FORK"
  LIST="LIST"
  GET="GET"
  SET="SET"
  REMOVE="REMOVE"
  END="END"
else
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
fi

# Regular expression to match variable assignment
stencila_assign_regex="^\s*((export|eval|(declare(\s+\-[-aAilnrtux])?))\s+)?[a-zA-Z_][a-zA-Z0-9_]*=.*$"

# SIGINT is handled while EXEC tasks are running but in case SIGINT is received just after a
# task finishes, or for some other reason inside the main loop, set trap to ignore it
trap "" SIGINT

# Print initial READY flag on stdout and stderr
printf "$READY\n" | tee /dev/stderr

# Read each stdin line as a task
while read -r stencila_line
do
  # Split the line into lines using the LINE flag as separator
  IFS="$LINE" read -ra stencila_lines <<< "$stencila_line"
  
  # Switch on the task flag (the first line)
  case "${stencila_lines[0]}" in 
    "$EXEC")
      for stencila_line in "${stencila_lines[@]}"; do
        if [[ "$stencila_line" =~ $stencila_assign_regex ]]; then
          stencila_assigns=true
          break
        fi
      done
      if [[ "$stencila_assigns" == true ]]; then
        # Execute remaining lines, at least one containing an assignment, here
        unset stencila_assigns
        eval $(printf "%s\n" "${stencila_lines[@]:1}")
      else
        # Execute remaining lines in background so the task can be interrupted
        eval $(printf "%s\n" "${stencila_lines[@]:1}") &
        trap "kill -SIGTERM $!" SIGINT
        wait $!
        trap "" SIGINT
      fi
      ;;
    "$EVAL")
      # Evaluate second line (integer expressions only)
      eval "echo $((${stencila_lines[1]}))"
      ;;
    "$LIST")
      # Return a list of variables (with limited type info and END flag after each)
      declare -p | sed -n 's/declare -\([-aAilnrtux]\) \([a-zA-Z_][a-zA-Z0-9_]*\)=\(.*\)/\1 \2/p' | while read -r stencila_options stencila_name; do
        if [[ $stencila_options == *"i"* ]]; then
          stencila_node_type="Integer"
          stencila_native_type="integer"
        elif [[ $stencila_options == *"a"* ]]; then
          stencila_node_type="Array"
          stencila_native_type="array"
        elif [[ $stencila_options == *"A"* ]]; then
          stencila_node_type="Object"
          stencila_native_type="associative array"
        else
          stencila_node_type="String"
          stencila_native_type="string"
        fi
        echo "{\"type\":\"Variable\",\"name\":\"$stencila_name\",\"nodeType\":\"$stencila_node_type\",\"nativeType\":\"$stencila_native_type\",\"programmingLanguage\":\"Bash\"} $END"
        unset stencila_options
        unset stencila_name
        unset stencila_node_type
        unset stencila_native_type
      done
      ;;
    "$GET")
      # Get a variable.
      # Note: special treatment for associative arrays is not supported.
      stencila_name="${stencila_lines[1]}"
      if [[ "$(declare -p "${stencila_name}" 2>/dev/null)" =~ "declare -a" ]]; then
        # Array: print as JSON array
        stencila_array="${stencila_name}[@]"
        declare -a stencila_array="(${!stencila_array})"
        if [[ "${#stencila_array[@]}" == "0" ]]; then
          printf "[]"
        else
          printf "[${stencila_array[0]}"
          for stencila_item in "${stencila_array[@]:1}"; do
            printf ",$stencila_item"
          done
          unset stencila_item
          printf "]"
        fi
        unset stencila_array
      else
        # Others: printf rather than echo to avoid trailing newline
        printf '%s' "${!stencila_name}"
      fi
      unset stencila_name
      ;;
    "$SET")
      # Set a variable. Uses -x option (for export) so that variables
      # are available to forks
      declare -x "${stencila_lines[1]}=${stencila_lines[2]}"
      ;;
    "$REMOVE")
      # Remove a variable
      unset "${stencila_lines[1]}"
      ;;
    "$FORK")
      # Fork the kernel instance and echo back the new pid
      bash "$0" 0<"${stencila_lines[1]}" 1>"${stencila_lines[2]}" 2>"${stencila_lines[3]}" &
      echo $!
      ;;
    *)
      echo "Unrecognised flag" >&2
      ;;
  esac
  
  # Print READY flag on stdout and stderr
  printf "$READY\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
