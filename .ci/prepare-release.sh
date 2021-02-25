#!/bin/sh

# Script to prepare a release by generating files before they are committed etc
# Used by @semantic-release/exec

VERSION=$1
echo "Preparing release $VERSION"

# Update the version in the top-level package
sed -i -e "s!\"version\": .*!\"version\": \"$VERSION\",!" package.json

# Update the version in the Node package
sed -i -e "s!\"version\": .*!\"version\": \"$VERSION\",!" node/package.json
sed -i -e "s!^version = .*!version = \"$VERSION\"!" node/native/Cargo.toml

# Update the version in the Python package
sed -i -e "s!^version = .*!version = \"$VERSION\"!" python/Cargo.toml

# Update the version in the R package
sed -i -e "s!^version = .*!version = \"$VERSION\"!" r/Cargo.toml

# Update the version in the Rust crate
sed -i -e "s!^version = .*!version = \"$VERSION\"!" rust/Cargo.toml

# Update the workspace Cargo.lock file so that above version changes
# are propagated to it 
cargo generate-lockfile
