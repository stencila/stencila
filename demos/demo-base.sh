# Source this script into your demo script

# Source demo-magic.sh and set some of it's options
. demo-magic.sh
DEMO_PROMPT=$BLUE"$ "
DEMO_CMD_COLOR=$GREEN

BOLD=$(tput bold)
NORMAL=$(tput sgr0)

# Functions to print headings
function h {
  echo -e ""
  p "# $BOLD$WHITE$1$COLOR_RESET$NORMAL"
  echo -e ""
}
function h1 {
  h "$1"
}
function h2 {
  h "$1"
}
function h3 {
  h "$1"
}

# Function to print a comment
function c {
  p "# $1"
}

# Function to execute a command
function e {
  pe "$1"
}

# Function to `sleep` when in non interactive mode i.e. `-n`
# and to continue when in interactive mode. This allows for pauses when
# recording a screencast but for them not to be there in interactive tutorials
function z {
  if $NO_WAIT; then
    sleep $1
  fi
}

clear
