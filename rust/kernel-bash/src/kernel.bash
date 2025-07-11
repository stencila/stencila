#!/usr/bin/env bash

# During development it can be useful to run this kernel script directly e.g.
#
#     DEV=true bash rust/kernel-bash/src/kernel.bash
#
# Use Ctrl+D to quit, since Ctrl+C is trapped for interrupting kernel tasks.
#
# IMPORTANT: When making changes to this file, especially adding or removing lines
# before the eval commands, make sure to update the stencila_base_line variables
# in the error processing section (around lines 102 and 105) to maintain correct
# line number calculation for error messages.

if [ "$DEV" == "true" ]; then
  READY="READY"
  LINE="|"
  EXEC="EXEC"
  EVAL="EVAL"
  INFO="INFO"
  PKGS="PKGS"
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
  INFO=$(printf "\U0010EE15")
  PKGS=$(printf "\U0010BEC4")
  LIST=$(printf "\U10C155")
  GET=$(printf "\U10A51A")
  SET=$(printf "\U107070")
  REMOVE=$(printf "\U10C41C")
  END=$(printf "\U10CB40")
fi

# Enable alias expansion in non-interactive shells
shopt -s expand_aliases

# Regular expression to match variable assignment or alias declaration
stencila_assign_regex="^\s*((export|eval|alias|(declare(\s+\-[-aAilnrtux])?))\s+)?[a-zA-Z_][a-zA-Z0-9_]*=.*$"

# Regular expression to match function declaration
stencila_function_regex="^\s*[a-zA-Z_][a-zA-Z0-9_]*\s*\(\s*\)\s*\{?\s*$"

# Define the 'print' function for outputting one or more Stencila Schema nodes
print() {
  for arg in "$@"; do
    printf "$arg$END\n"
  done
}

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
      for stencila_line in "${stencila_lines[@]:1}"; do
        if [[ "$stencila_line" =~ $stencila_assign_regex ]] || [[ "$stencila_line" =~ $stencila_function_regex ]]; then
          stencila_assigns=true
          break
        fi
      done
      printf -v stencila_code "%s\n" "${stencila_lines[@]:1}"
      # Create a temporary file for stderr
      stencila_stderr_tmp=$(mktemp)
      if [[ "$stencila_assigns" == true ]]; then
        # Execute remaining lines, at least one containing an assignment or function declaration, here
        eval "$stencila_code" 2>"$stencila_stderr_tmp"
        stencila_exit_code=$?
      else
        # Execute remaining lines in background so the task can be interrupted
        eval "$stencila_code" 2>"$stencila_stderr_tmp" &
        trap "kill -SIGTERM $!" SIGINT
        wait $!
        stencila_exit_code=$?
        trap "" SIGINT
      fi
      # Process and output any stderr content as JSON
      if [[ -s "$stencila_stderr_tmp" ]]; then
        # Count the number of lines in the code to help with line number adjustment
        stencila_code_lines=$(echo -n "$stencila_code" | grep -c '^' || echo 1)
        # Determine the message level based on exit code
        if [[ $stencila_exit_code -ne 0 ]]; then
          stencila_message_level="Error"
        else
          stencila_message_level="Info"
        fi
        
        # Read entire stderr content
        stencila_stderr_content=$(<"$stencila_stderr_tmp")
        
        # Check if this is a bash error with line number
        # Handle various error formats:
        # - /path/to/script: line 123: error message
        # - /path/to/script: eval: line 123: error message  
        # - line 123: error message
        if [[ "$stencila_stderr_content" =~ :\ line\ ([0-9]+):\ (.+) ]] || [[ "$stencila_stderr_content" =~ ^line\ ([0-9]+):\ (.+) ]]; then
          # This is a bash error - extract line number and message
          stencila_kernel_line="${BASH_REMATCH[1]}"
          stencila_error_msg="${BASH_REMATCH[2]}"
          
          # For multi-line code, bash counts from the eval line
          # We need to calculate the offset based on where eval was called
          # and adjust for the actual line in user's code
          if [[ "$stencila_assigns" == true ]]; then
            # eval is on line 83
            stencila_base_line=83
          else
            # eval is on line 87
            stencila_base_line=87
          fi
          # Calculate user line: kernel_line - base_line (0-based)
          stencila_user_line=$((stencila_kernel_line - stencila_base_line))
          # Ensure line number is within valid range (0-based)
          if [[ $stencila_user_line -lt 0 ]]; then
            stencila_user_line=0
          elif [[ $stencila_user_line -ge $stencila_code_lines ]]; then
            stencila_user_line=$((stencila_code_lines - 1))
          fi
          # Escape quotes and newlines for JSON
          stencila_error_msg="${stencila_error_msg//\\/\\\\}"
          stencila_error_msg="${stencila_error_msg//\"/\\\"}"
          stencila_error_msg="${stencila_error_msg//$'\n'/\\n}"
          printf '{"type":"ExecutionMessage","level":"%s","message":"%s","codeLocation":{"type":"CodeLocation","startLine":%d}}%s\n' \
            "$stencila_message_level" "$stencila_error_msg" "$stencila_user_line" "$END" >&2
        else
          # Not a bash error - output the entire message as-is
          # Escape quotes and newlines for JSON
          stencila_stderr_content="${stencila_stderr_content//\\/\\\\}"
          stencila_stderr_content="${stencila_stderr_content//\"/\\\"}"
          stencila_stderr_content="${stencila_stderr_content//$'\n'/\\n}"
          printf '{"type":"ExecutionMessage","level":"%s","message":"%s"}%s\n' \
            "$stencila_message_level" "$stencila_stderr_content" "$END" >&2
        fi
      fi
      rm -f "$stencila_stderr_tmp"
      unset stencila_assigns stencila_code stencila_stderr_tmp stencila_error_line stencila_kernel_line stencila_user_line stencila_code_lines stencila_base_line stencila_error_msg stencila_exit_code stencila_message_level BASH_REMATCH
      ;;
    "$EVAL")
      # Evaluate second line (integer expressions only)
      eval "echo $((${stencila_lines[1]}))"
      ;;
    "$INFO")
      # Return runtime information
      echo "{\"type\":\"SoftwareApplication\",\"name\":\"Bash\",\"softwareVersion\":\"$BASH_VERSION\",\"operatingSystem\":\"$(uname -s)\"}"
      ;;
    "$PKGS")
      # Return an empty list of packages
      ;;
    "$LIST")
      # Return a list of variables (with limited type info and END flag after each)
      declare -p | sed -n 's/declare -\([-aAilnrtux]\) \([a-zA-Z_][a-zA-Z0-9_]*\)=\(.*\)/\1 \2/p' | while read -r stencila_options stencila_name; do
        if [[ $stencila_name == stencila_* ]]; then
          continue
        fi
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
        unset stencila_options stencila_name stencila_node_type stencila_native_type
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
      unset stencila_name BASH_REMATCH
      ;;
    "$SET")
      # Set a variable.
      declare "${stencila_lines[1]}=${stencila_lines[2]}"
      ;;
    "$REMOVE")
      # Remove a variable
      unset "${stencila_lines[1]}"
      ;;
    *)
      echo "Unrecognised flag" >&2
      ;;
  esac
  
  # Print READY flag on stdout and stderr
  printf "$READY\n" | tee /dev/stderr
done < "${1:-/dev/stdin}"
