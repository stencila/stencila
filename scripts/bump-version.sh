#!/bin/bash

# Script to create a new tag and bump the version of all
# products in this repo to the new tag

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
    echo "Bumping versions to $VERSION"
fi

# Update the version in the Rust server and CLI crates
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/server/Cargo.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" rust/cli/Cargo.toml

# Update the version in the TypeScript package
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" ts/package.json

# Update the version of `@stencila/types` and `@stencila/node` in the package.json of the latter
sed -i -e "s/    \"@stencila\/types\": .*/    \"@stencila\/types\": \"$VERSION\"/" node/stencila-node/package.json
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" node/stencila-node/package.json
sed -i -e "s/^version = .*/version = \"$VERSION\"/" node/stencila-node/Cargo.toml

# Update the version `@stencila/types` and `@stencila/web` in the package.json of the latter
sed -i -e "s/    \"@stencila\/types\": .*/    \"@stencila\/types\": \"$VERSION\",/" web/package.json
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" web/package.json

# Do NPM install at root to update `package-lock.json` files
# but with --ignore-scripts to avoid a premature attempt to download the
# as yet unavailable binary addons
npm install --ignore-scripts

# Update the workspace Cargo.lock file so that above version changes
# are propagated to it 
cargo generate-lockfile

# Commit the changes files
git add .
git commit -m "chore(*): Version $VERSION"

# Create the tag
git tag "v$VERSION"

echo "Version bumped and tag created"
echo "Now 'git push && git push --tags'"
