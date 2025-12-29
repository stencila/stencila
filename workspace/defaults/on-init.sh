#!/usr/bin/env bash

# Runs when a workspace is created.
# Initializes a Stencila workspace in the local Git repository.

set -euo pipefail

echo "⚙️ Initializing Stencila workspace..."

stencila init --yes

# Check if there are changes to commit and push
if [[ -n "$(git status --porcelain)" ]]; then
    echo "📝 Committing and pushing initialization changes..."

    # Add all changes and commit
    git add -A
    git commit -m "Initialize Stencila workspace"
    echo "✅ Changes committed"

    # Push to remote repository
    echo "🚀 Pushing changes to remote repository..."
    git push origin HEAD
else
    echo "no_changes" > /tmp/stencila-status
    echo "ℹ️  No changes to commit"
fi

echo "✨ Stencila workspace initialization completed successfully!"
