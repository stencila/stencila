#!/bin/bash

# Script used to tell if a folder has changes in it compared to a base.
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
# instead of `git branch` to determine if on `main`.
HEAD=$(git rev-parse HEAD)
MAIN=$(git rev-parse origin/main)
if [[ $HEAD == $MAIN ]]; then
    DOTS=".."
    BASE=$(git describe --tags --abbrev=0)
else
    DOTS="..."
    BASE=$MAIN
fi

git diff --quiet $BASE$DOTS$HEAD -- $1
if [ $? -eq 1 ]; then
    echo "Changes to $1 since last tag"
    exit 1
else
    echo "No changes to $1 since last tag"
    exit 0
fi
