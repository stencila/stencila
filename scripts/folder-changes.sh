#!/bin/bash

# Script used to create a list of folders that have changes in
# them compared to a base.
#
# If on `main` branch then the latest tag is the base.
# If on another branch then `main` branch is the base.
#
# If on `main` then use the default two dots.
# On a branch used three dots so that get the differences
# starting at the last common commit (in case `main`` has had commits
# in the meantime).
# See https://matthew-brett.github.io/pydagogue/git_diff_dots.html

# On CI, we may be in "detached HEAD" state so use `git rev-parse`
# instead of `git branch` to determine if on `main``.
HEAD=$(git rev-parse HEAD)
MASTER=$(git rev-parse origin/main)
if [[ $HEAD == $MASTER ]]; then
    DOTS=".."
    BASE=$(git describe --tags --abbrev=0)
    echo "On main ($MASTER), comparing to tag $BASE using '$DOTS'"
else
    DOTS="..."
    BASE=$MASTER
    echo "On branch ($HEAD), comparing to main ($BASE) using '$DOTS'"
fi

CHANGED=""
for FOLDER in docker docs node python r rust schema web; do
    git diff --quiet $BASE$DOTS$HEAD -- $FOLDER
    if [ $? -eq 1 ]; then
        CHANGED="$CHANGED$FOLDER,"
    fi
done

echo "Folders with changes: $CHANGED"
echo "##vso[task.setvariable variable=changed;isOutput=true]$CHANGED"
