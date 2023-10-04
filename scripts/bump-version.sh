#!/bin/bash

# Script to create a new tag and bump the version of all
# products in this repo to the new tag

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
    echo "Bumping versions to $VERSION"
fi

# Update the version in the Rust CLI
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/cli/Cargo.toml

# Update the version in the Typescript & Node.js SDK
npm version $VERSION --workspaces
sed -i -e "s/^version = .*/version = \"$VERSION\"/" node/Cargo.toml

# Update the version in the Python SDK
sed -i -e "s/^version = .*/version = \"$VERSION\"/" python/Cargo.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" python/pyproject.toml
(cd python && poetry install)

# Update the workspace Cargo.lock file so that above version changes
# are propagated to it 
cargo generate-lockfile

# Commit the changes files
git add .
git commit -m "chore(*): Bump version\n\n[skip ci]"

# Create the tag
git tag "v$VERSION"

echo "Version bumped and tag created"
echo "Now 'git push && git push --tags'"
