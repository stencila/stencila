#!/bin/bash

# Script for generating Pandoc JSON fixtures from Markdown files.
# Uses `jq` to format the JSON for easier development and debugging.
#
# Usage
#   # All .md files in ../md
#   ./pandoc.sh all
#
#   A specific file
#   ./pandoc.sh input.md output.json

one () {
    echo "Converting $1 to $2"
    pandoc $1 --to json | jq . | tee $2 > /dev/null
}

all () {
    for FILE in $(ls ../md)
    do
        one ../md/$FILE "${FILE%.*}.json"
    done
}

if [[ $1 == "all" ]]
then
  all
else
  one $1 $2
fi
