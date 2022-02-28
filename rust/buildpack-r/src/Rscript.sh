#!/usr/bin/env bash
# Why? See lib.rs
set -e
R="R --slave --no-restore"
case "$1" in
    "-e") $R -e "$2" ;;
    *) $R --file="$1" ;;
esac
