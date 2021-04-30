#!/bin/sh

# Script used to create a list of folders that have changes in
# them compared to a base.
#
# If on `master` branch then the latest tag is the base.
# If on another branch then `master` branch is the base.
#
# Used by `azure-pipelines.yml`.

# On CI, we may be in "detached HEAD" state so
# use `git rev-parse` instead of `git branch` to
# determin if on master.
HEAD=$(git rev-parse HEAD)
MASTER=$(git rev-parse origin/master)
if [[ $HEAD == $MASTER ]]; then
    BASE=$(git describe --tags --abbrev=0)
else
    BASE=$MASTER
fi
echo "Comparing against: $BASE"

CHANGED=""
for FOLDER in cli desktop docker help node rust; do
    git diff --quiet HEAD $BASE -- $FOLDER
    if [ $? -eq 1 ]; then
        CHANGED="$CHANGED$FOLDER,"
    fi
done

echo "Folders with changes: $CHANGED"
echo "##vso[task.setvariable variable=changed;isOutput=true]$CHANGED"
