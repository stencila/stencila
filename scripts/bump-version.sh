#!/bin/bash

# Script to create a new tag and bump the version of all
# products in this repo to the new tag

VERSION=$1
SEMVER="^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(\-[0-9A-Za-z\-\.]+)?(\+[0-9A-Za-z\-\.]+)?$"

if ! [[ "$VERSION" =~ $SEMVER ]]; then
    echo "Version argument should be a semantic version number; got $VERSION"
    exit 1
else
    echo "Bumping versions to $VERSION"
fi

# Create the tag
git tag "v$VERSION"

# Update the version in the Rust CLI
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/cli/Cargo.toml

# Update the version in the Typescript package
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" typescript/package.json
(cd typescript && npm install)

# Update the version in the Node.js SDK
sed -i -e "s/^version = .*/version = \"$VERSION\"/" node/Cargo.toml
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" node/package.json
(cd node && npm install)

# Update the version in the Python SDK
sed -i -e "s/^version = .*/version = \"$VERSION\"/" python/Cargo.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" python/pyproject.toml
(cd python && poetry install)

# Update the workspace Cargo.lock file so that above version changes
# are propagated to it 
cargo generate-lockfile
