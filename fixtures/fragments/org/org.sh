#!/bin/bash

# Script for generating Org examples from Markdown examples.
# Usage
#   # All .md files in ../md
#   ./org.sh all
#
#   A specific file
#   ./org.sh input.md output.json

one () {
    echo "Converting $1 to $2"
    pandoc $1 --to org | tee $2 > /dev/null
}

all () {
    for FILE in $(ls ../md)
    do
        one ../md/$FILE "${FILE%.*}.org"
    done
}

if [[ $1 == "all" ]]
then
  all
else
  one $1 $2
fi
