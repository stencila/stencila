#!/bin/bash

# Script to create a new tag and bump the version of the CLI
# Creates a new tag for the version

set -e

VERSION=$1
SEMVER="^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(\-[0-9A-Za-z\-\.]+)?(\+[0-9A-Za-z\-\.]+)?$"

if [[ -n $(git status -s) ]]; then
    echo 'Modified and/or untracked files! Commit or stash first.'
    exit 1
fi

if ! [[ "$VERSION" =~ $SEMVER ]]; then
    echo "Version argument should be a semantic version number; got $VERSION"
    exit 1
else
    echo "Bumping CLI to $VERSION"
fi

# Update the version in the Rust version crate
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/version/Cargo.toml

# Build the version crates so that change is propagated to the Cargo.lock file
cargo build -p version

# Commit the changed files
git add .
git commit -m "release(cli): v$VERSION"

# Create the tag
git tag "v$VERSION"

echo "Version bumped and tag created"
echo "Now 'git push && git push --tags'"
