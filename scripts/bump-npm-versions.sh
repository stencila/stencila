#!/bin/bash

# Script to bump the versions of the NPM packages in the `node`,
# `ts`, and `web` subdirectories.

set -e

if [ $# -lt 4 ]; then
    echo "Error: Not enough arguments provided."
    echo
    echo "Provide semantic version numbers, in the following order, for"
    echo "'@stencila/types', '@stencila/plugin', '@stencila/node', and '@stencila/web'."
    echo
    echo "Usage: $0 X.X.X Y.Y.Y Z.Z.Z ..."
    exit 1
fi

SEMVER="^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(\-[0-9A-Za-z\-\.]+)?(\+[0-9A-Za-z\-\.]+)?$"
for arg in "$@"; do
    if ! [[ "$arg" =~ $SEMVER ]]; then
        echo "Arguments should be semantic version numbers; got $arg"
        exit 1
    fi
done

STENCILA_TYPES_VERSION=$1
STENCILA_PLUGIN_VERSION=$2
STENCILA_NODE_VERSION=$3
STENCILA_WEB_VERSION=$3

echo "🏗️ Bumping:"
echo " @stencila/types to $STENCILA_TYPES_VERSION"
echo " @stencila/plugin to $STENCILA_PLUGIN_VERSION"
echo " @stencila/node to $STENCILA_NODE_VERSION"
echo " @stencila/web to $STENCILA_WEB_VERSION"
echo

# Update the version of @stencila/types`
sed -i -e "s/\"version\": .*/\"version\": \"$STENCILA_TYPES_VERSION\",/" ts/package.json

# Update the version of `@stencila/types` and `@stencila/plugin` in the package.json of the latter
sed -i -e "s/    \"@stencila\/types\": .*/    \"@stencila\/types\": \"$STENCILA_TYPES_VERSION\"/" node/stencila-plugin/package.json
sed -i -e "s/\"version\": .*/\"version\": \"$STENCILA_PLUGIN_VERSION\",/" node/stencila-plugin/package.json

# Update the version of `@stencila/types` and `@stencila/node` in the package.json of the latter
sed -i -e "s/    \"@stencila\/types\": .*/    \"@stencila\/types\": \"$STENCILA_TYPES_VERSION\"/" node/stencila-node/package.json
sed -i -e "s/\"version\": .*/\"version\": \"$STENCILA_NODE_VERSION\",/" node/stencila-node/package.json
sed -i -e "s/^version = .*/version = \"$STENCILA_NODE_VERSION\"/" node/stencila-node/Cargo.toml

# Update the version `@stencila/types` and `@stencila/web` in the package.json of the latter
sed -i -e "s/    \"@stencila\/types\": .*/    \"@stencila\/types\": \"$STENCILA_TYPES_VERSION\",/" web/package.json
sed -i -e "s/\"version\": .*/\"version\": \"$STENCILA_WEB_VERSION\",/" web/package.json

# Do NPM install at root to update `package-lock.json` files
# but with --ignore-scripts to avoid a premature attempt to download the
# as yet unavailable binary addons
npm install --ignore-scripts

# Update the workspace Cargo.lock file so that above version change is propagated to it 
cargo generate-lockfile

echo "🏁 Updated versions"
echo "Check changes to package.json, and other, files and commit"
