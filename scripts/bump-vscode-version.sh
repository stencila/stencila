#!/bin/bash

# Script to bump the versions of the VSCode extension.

set -e

VERSION=$1
SEMVER="^(0|[1-9]\d*)\.(0|[1-9]\d*)\.([0-9]+)$"

if ! [[ "$VERSION" =~ $SEMVER ]]; then
    echo "Version argument should be a semantic version number; got $VERSION"
    exit 1
else
    echo "Bumping VSCode extension to $VERSION"
fi

cd vscode

# Update the version in `package.json`
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" package.json

# Do NPM install to update `package-lock.json` with new version
npm install

# Create CHANGELOG.md entry
sed -i "3i## $VERSION $(date '+%Y-%m-%d')\n\n\n" CHANGELOG.md

echo "🏁 Updated version"
echo "Check update install-cli.sh if needs be, changes to package.json and other files, add details to the CHANGELOG.md and commit"
