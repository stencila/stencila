#!/usr/bin/env bash

set -euo pipefail

# sync-from-remote.sh
# Syncs content from remote cloud services to the local Git repository using Stencila CLI

echo "🔄 Starting sync from remote..."

# Validate required environment variables
if [[ -z "${GITHUB_REPO:-}" ]]; then
    echo "❌ Error: GITHUB_REPO environment variable is required"
    exit 1
fi

if [[ -z "${STENCILA_SYNC_FILE_PATH:-}" ]]; then
    echo "❌ Error: STENCILA_SYNC_FILE_PATH environment variable is required"
    exit 1
fi

if [[ -z "${STENCILA_SYNC_REMOTE_URL:-}" ]]; then
    echo "❌ Error: STENCILA_SYNC_REMOTE_URL environment variable is required"
    exit 1
fi

# Navigate to the repository directory
REPO_DIR="/home/workspace/${GITHUB_REPO}"
if [[ ! -d "${REPO_DIR}" ]]; then
    echo "❌ Error: Repository directory not found: ${REPO_DIR}"
    exit 1
fi

cd "${REPO_DIR}"
echo "📁 Working directory: $(pwd)"
echo "📄 File path: ${STENCILA_SYNC_FILE_PATH}"
echo "☁️  Remote URL: ${STENCILA_SYNC_REMOTE_URL}"

# Pull from remote using Stencila CLI
echo "⬇️  Pulling from remote..."
stencila pull "${STENCILA_SYNC_FILE_PATH}" "${STENCILA_SYNC_REMOTE_URL}"

# Check if there are changes to commit and push
if [[ -n "$(git status --porcelain)" ]]; then
    echo "📝 Committing and pushing changes from sync..."

    # Configure git if not already configured
    if [[ -z "$(git config --get user.email || true)" ]]; then
        git config user.name "Stencila User"
        git config user.email "noreply@stencila.cloud"
    fi

    # Add all changes
    git add -A

    # Create commit
    COMMIT_MSG="Sync from remote [skip ci]"
    git commit -m "${COMMIT_MSG}"
    echo "✅ Changes committed: ${COMMIT_MSG}"

    # Push to remote repository
    echo "🚀 Pushing changes to remote repository..."
    git push --set-upstream origin HEAD
    echo "✅ Changes pushed to remote repository"
else
    echo "ℹ️  No changes to commit"
fi

echo "✨ Sync from remote completed successfully!"
exit 0
