#!/bin/bash

# Unified script to bump the version across all Stencila components
# Updates CLI, VSCode extension, npm packages, and Python packages to a single unified version

set -e

VERSION=$1
SEMVER="^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$"

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

# 0. Ensure generated files are up-to-date
echo "Checking generated files are up-to-date..."
make -C rust generated

if [[ -n $(git status -s) ]]; then
    echo ""
    echo "⚠️  Generated files are out of date. The following files were modified:"
    git status -s
    echo ""
    read -p "Would you like to commit these changes now? [y/N] " -r REPLY
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git add .
        git commit -m "chore(*): update generated files"
        echo "✅ Generated files committed."
    else
        echo "Aborting release. Please commit or stash the generated file changes first."
        exit 1
    fi
fi

# 1. Update workspace version in root Cargo.toml (source of truth)
echo "Updating root Cargo.toml..."
sed -i -e "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# 2. Update VSCode extension
echo "Updating VSCode extension..."
sed -i -e "s/\"version\": .*/\"version\": \"$VERSION\",/" vscode/package.json
sed -i -e "s/^VERSION=.*/VERSION=\"v$VERSION\"/" vscode/install-cli.sh
npm install --package-lock-only --prefix vscode

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
# Building the version crate is sufficient to update Cargo.lock with the new version
# Note: DO NOT use `cargo generate-lockfile` as it would update ALL dependencies to their
# latest versions, which is not desired during a version bump
cargo build -p stencila-version
npm install

# 7. Update VSCode CHANGELOG
echo "Updating VSCode CHANGELOG..."
cd vscode
sed -i "3i## $VERSION $(date '+%Y-%m-%d')\n\n\n" CHANGELOG.md
cd ..

# 8. Update main CHANGELOG
echo "Updating main CHANGELOG..."
# Get the previous version tag for the compare link
PREV_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "vX.X.X")
sed -i "1i# [$VERSION](https://github.com/stencila/stencila/compare/$PREV_VERSION...v$VERSION) ($(date '+%Y-%m-%d'))\n\n\n### Bug Fixes\n\n\n### Features\n\n\n" CHANGELOG.md

# 9. Prompt user to update changelogs before committing
echo ""
echo "✏️  Please update the following changelogs with release notes for v$VERSION:"
echo "   - CHANGELOG.md (main changelog with categorized sections)"
echo "   - vscode/CHANGELOG.md (VSCode extension changelog)"
echo ""
echo "   Tips for writing release notes:"
echo "   - Focus on user-facing changes and highlights, not every commit"
echo "   - Group related changes into paragraphs rather than listing individually"
echo "   - Emphasize new features, important fixes, and breaking changes"
echo "   - Use 'git log --oneline $PREV_VERSION..HEAD' to review commits"
echo ""
read -p "Press Enter when you have updated the changelogs and are ready to commit..." -r
echo ""

# 10. Commit and tag
echo "Creating git commit and tag..."
git add .
git commit -m "release: v$VERSION"
git tag "v$VERSION"

echo ""
echo "✅ Version bumped to $VERSION across all components!"
echo ""
echo "Updated:"
echo "  - CLI (Rust workspace)"
echo "  - Main CHANGELOG.md"
echo "  - VSCode extension (including vscode/CHANGELOG.md)"
echo "  - npm packages (@stencila/types, @stencila/node, @stencila/web)"
echo "  - Python packages (stencila_types, stencila)"
echo ""
echo "Next steps:"
echo "  1. Review the commit: git show"
echo "  2. Push the release: git push && git push --tags"
