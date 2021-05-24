#!/bin/bash

# Script to prepare a release by generating files before they are committed etc
# Used by @semantic-release/exec

VERSION=$1
echo "Preparing release $VERSION"

# Note that the version in the top level package.json gets updated
# by the "@semantic-release/npm" so does not need to be dealt with here.

# Update the version in the Python package
sed -i -e "s!^    version=.*!    version=\"$VERSION\",!" py/setup.py

# Update the version in the R package
sed -i -e "s!^Version:.*!Version: $VERSION!" r/DESCRIPTION

# Update the version in the Rust crate (including lock file)
sed -i -e "s!^version = .*!version = \"$VERSION\"!" rs/Cargo.toml
(cd rust && cargo generate-lockfile)
