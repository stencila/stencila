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

# Update the version in the Rust server and CLI crates
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/server/Cargo.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/cli/Cargo.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/codec-json/Cargo.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/codec-yaml/Cargo.toml

# Update the workspace Cargo.lock file so that above version changes
# are propagated to it 
cargo generate-lockfile

# Commit the changed files
git add .
git commit -m "chore(*): Version $VERSION"

# Create the tag
git tag "v$VERSION"

echo "Version bumped and tag created"
echo "Now 'git push && git push --tags'"
