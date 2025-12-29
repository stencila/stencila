#!/usr/bin/env bash

# Syncs content from remote cloud services to the local Git repository using Stencila CLI

set -euo pipefail

# Validate required environment variables
if [[ -z "${STENCILA_SYNC_FILE_PATH:-}" ]]; then
    echo "❌ Error: STENCILA_SYNC_FILE_PATH environment variable is required"
    exit 1
fi

if [[ -z "${STENCILA_SYNC_REMOTE_URL:-}" ]]; then
    echo "❌ Error: STENCILA_SYNC_REMOTE_URL environment variable is required"
    exit 1
fi

echo "⬇️  Pulling ${STENCILA_SYNC_FILE_PATH} from ${STENCILA_SYNC_REMOTE_URL} ..."

# Pull from remote
# Use --no-merge to simply convert the downloaded document and
# avoid creating a new branch (because already on branch when syncing from remote)
stencila pull --no-merge "${STENCILA_SYNC_FILE_PATH}" --from "${STENCILA_SYNC_REMOTE_URL}"

# Check if there are changes to commit and push
if [[ -n "$(git status --porcelain)" ]]; then
    echo "📝 Committing and pushing changes from sync..."

    # Add all changes and commit
    git add -A
    git commit -m "Sync from remote [skip ci]"
    echo "✅ Changes committed"

    # Push to remote repository
    echo "🚀 Pushing changes to remote repository..."
    git push -u origin HEAD
    echo "✅ Changes pushed to remote repository"
else
    echo "no_changes" > /tmp/stencila-status
    echo "ℹ️  No changes to commit"
fi

echo "✨ Sync from remote completed successfully!"
