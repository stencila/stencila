#!/bin/bash

# Script to create an archive for a Rust binary

PREFIX=$1
TARGET=$2
BINARY=$3

VERSION=$(git describe --abbrev=0 --tags)

echo "Archiving target/$TARGET/release/$BINARY to $PREFIX-$VERSION-$TARGET.$EXT"

DIR="$PREFIX-$VERSION-$TARGET"

cd target
mkdir -p "$DIR"
cp ../LICENSE "$DIR"

if [[ "$(uname)" == *CYGWIN* || "$(uname)" == *MINGW* ]]; then
    cp "$TARGET/release/$BINARY.exe" "$DIR"
    7z a "$DIR.zip" "$DIR"
else
    cp "$TARGET/release/$BINARY" "$DIR"
    tar -cJf "$DIR.tar.xz" "$DIR"
fi;
