#!/bin/bash

# Script used to create a list of folders that have changes in
# them compared to a base.
#
# If on `master` branch then the latest tag is the base.
# If on another branch then `master` branch is the base.
#
# If on `master` then use the default two dots.
# On a branch used three dots so that get the differences
# starting at the last common commit (in case master has had commits
# in the meantime).
# See https://matthew-brett.github.io/pydagogue/git_diff_dots.html
#
# Used by `azure-pipelines.yml`.

# On CI, we may be in "detached HEAD" state so
# use `git rev-parse` instead of `git branch` to
# determine if on master.
HEAD=$(git rev-parse HEAD)
MASTER=$(git rev-parse origin/master)
if [[ $HEAD == $MASTER ]]; then
    DOTS=".."
    BASE=$(git describe --tags --abbrev=0)
    echo "On master ($MASTER), comparing to tag $BASE using '$DOTS'"
else
    DOTS="..."
    BASE=$MASTER
    echo "On branch ($HEAD), comparing to master ($BASE) using '$DOTS'"
fi

CHANGED=""
for FOLDER in cli desktop docker help node rust; do
    git diff --quiet $BASE$DOTS$HEAD -- $FOLDER
    if [ $? -eq 1 ]; then
        CHANGED="$CHANGED$FOLDER,"
    fi
done

echo "Folders with changes: $CHANGED"
echo "##vso[task.setvariable variable=changed;isOutput=true]$CHANGED"
