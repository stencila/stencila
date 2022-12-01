#!/bin/bash

# Script to prepare a release by generating files before they are committed etc
# Used by @semantic-release/exec

VERSION=$1
echo "Preparing release $VERSION"

# Update the version in the top-level package
sed -i -e "s!\"version\": .*!\"version\": \"$VERSION\",!" package.json

# Update the versions in the Node, Typescript and Web packages and do npm install
# to propagate change to package-lock.json files
for FOLDER in node ts web; do
    # sed -i -e "s!\"version\": .*!\"version\": \"$VERSION\",!" node/package.json
    # sed -i -e "s!^version = .*!version = \"$VERSION\"!" node/Cargo.toml
    # (cd node && npm install)
done

# Update the version in the Python package
# sed -i -e "s!^version = .*!version = \"$VERSION\"!" python/Cargo.toml
# sed -i -e "s!^    version=.*!    version=\"$VERSION\",!" python/setup.py

# Update the version in the R package
# sed -i -e "s!^version = .*!version = \"$VERSION\"!" r/Cargo.toml
# sed -i -e "s!^Version:.*!Version: $VERSION!" r/DESCRIPTION

# Update the version in the Rust crates (including lock file)
sed -i -e "s!^version = .*!version = \"$VERSION\"!" rust/stencila/Cargo.toml
(cd rust && cargo generate-lockfile)

# Update the workspace Cargo.lock file so that above version changes
# are propagated to it 
cargo generate-lockfile
