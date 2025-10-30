#!/bin/bash

# Unified script to bump the version across all Stencila components
# Updates CLI, VSCode extension, npm packages, and Python packages to a single unified version

set -e

VERSION=$1
SEMVER="^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$"

# Validate inputs
if [[ -n $(git status -s) ]]; then
    echo 'Modified and/or untracked files! Commit or stash first.'
    exit 1
fi

if ! [[ "$VERSION" =~ $SEMVER ]]; then
    echo "Version argument should be a semantic version number (stable versions only); got $VERSION"
    exit 1
else
    echo "Bumping all components to version $VERSION"
fi

# 1. Update workspace version in root Cargo.toml (source of truth)
echo "Updating root Cargo.toml..."
sed -i -e "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# 2. Update VSCode extension
echo "Updating VSCode extension..."
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" vscode/package.json
sed -i -e "s/^VERSION=.*/VERSION=\"v$VERSION\"/" vscode/install-cli.sh

# 3. Update npm packages
echo "Updating npm packages..."
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" ts/package.json
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" node/package.json
sed -i -e "s/^version = .*/version = \"$VERSION\"/" node/Cargo.toml
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" web/package.json

# 4. Update Python packages
echo "Updating Python packages..."
sed -i -e "s/^version = .*/version = \"$VERSION\"/" python/stencila_types/pyproject.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" python/stencila/pyproject.toml
sed -i -e "s/^version = .*/version = \"$VERSION\"/" python/stencila/Cargo.toml

# 5. Rename schema snapshot and migration files
SCHEMAS_DIR="rust/node-db/schemas"
MIGRATIONS_DIR="rust/node-db/migrations"

if [ -f "$SCHEMAS_DIR/v99.99.99.json" ]; then
    echo "Renaming schema snapshot v99.99.99.json to v$VERSION.json"
    mv "$SCHEMAS_DIR/v99.99.99.json" "$SCHEMAS_DIR/v$VERSION.json"
fi

if [ -f "$MIGRATIONS_DIR/v99.99.99.cypher" ]; then
    echo "Renaming migration v99.99.99.cypher to v$VERSION.cypher"
    mv "$MIGRATIONS_DIR/v99.99.99.cypher" "$MIGRATIONS_DIR/v$VERSION.cypher"
fi

# 6. Build and update lock files
echo "Building version crate and updating lock files..."
cargo build -p stencila-version
npm install
cargo generate-lockfile

# 7. Update VSCode CHANGELOG
echo "Updating VSCode CHANGELOG..."
cd vscode
sed -i "3i## $VERSION $(date '+%Y-%m-%d')\n\n\n" CHANGELOG.md
cd ..

# 8. Commit and tag
echo "Creating git commit and tag..."
git add .
git commit -m "release: v$VERSION"
git tag "v$VERSION"

echo ""
echo "✅ Version bumped to $VERSION across all components!"
echo ""
echo "Updated:"
echo "  - CLI (Rust workspace)"
echo "  - VSCode extension"
echo "  - npm packages (@stencila/types, @stencila/node, @stencila/web)"
echo "  - Python packages (stencila_types, stencila)"
echo ""
echo "Next steps:"
echo "  1. Update vscode/CHANGELOG.md with release notes"
echo "  2. Review changes: git show"
echo "  3. Push: git push && git push --tags"
