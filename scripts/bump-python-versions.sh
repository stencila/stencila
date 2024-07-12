#!/bin/bash

# Script to bump the versions of the Python packages in the `python`
# subdirectory.

set -e

if [ $# -lt 3 ]; then
    echo "Error: Not enough arguments provided."
    echo
    echo "Provide semantic version numbers, in the following order, for"
    echo "'stencila_types', 'stencila_plugin', and 'stencila'."
    echo
    echo "Usage: $0 X.X.X Y.Y.Y Z.Z.Z"
    exit 1
fi

SEMVER="^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)((a|b)[0-9]+)?$"
for arg in "$@"; do
    if ! [[ "$arg" =~ $SEMVER ]]; then
        echo "Arguments should be semantic version numbers; got $arg"
        exit 1
    fi
done

STENCILA_TYPES_VERSION=$1
STENCILA_PLUGIN_VERSION=$2
STENCILA_VERSION=$3

echo "🏗️ Bumping:"
echo " python/stencila_types to $STENCILA_TYPES_VERSION"
echo " python/stencila_plugin to $STENCILA_PLUGIN_VERSION"
echo " python/stencila to $STENCILA_VERSION"
echo

# Update the versions in python/stencila_types
sed -i -e "s/^version = .*/version = \"$STENCILA_TYPES_VERSION\"/" python/stencila_types/pyproject.toml

# Update the versions in python/stencila_plugin
sed -i -e "s/stencila-types.*/stencila-types==$STENCILA_TYPES_VERSION\",/" python/stencila_plugin/pyproject.toml
sed -i -e "s/^version = .*/version = \"$STENCILA_PLUGIN_VERSION\"/" python/stencila_plugin/pyproject.toml

# Update the versions in python/stencila (Python SDK)
sed -i -e "s/stencila-types.*/stencila-types==$STENCILA_TYPES_VERSION\",/" python/stencila/pyproject.toml
sed -i -e "s/^version = .*/version = \"$STENCILA_VERSION\"/" python/stencila/pyproject.toml
sed -i -e "s/^version = .*/version = \"${STENCILA_VERSION//b/-beta.}\"/" python/stencila/Cargo.toml

# Update the workspace Cargo.lock file so that above version change is propagated to it 
cargo generate-lockfile

echo "🏁 Updated versions"
echo "Check changes to pyproject.toml and other files and commit"
