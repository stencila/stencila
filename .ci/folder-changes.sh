#!/bin/sh

# Script used to create a list of folders that have changes in
# them since the last tag.
# Used by `azure-pipelines.yml`.

PREV=$(git describe --tags --abbrev=0)

CHANGED=""
for FOLDER in cli desktop docker help node rust
do
    git diff --quiet HEAD $PREV -- $FOLDER
    if [ $? -eq 1 ]; then
        CHANGED="$CHANGED$FOLDER,"
    fi
done

echo "Folders with changes: $CHANGED"
echo "##vso[task.setvariable variable=changed;isOutput=true]$CHANGED"
