#!/usr/bin/env bash

set -euo pipefail

# sync-to-remote.sh
# Syncs content from the local Git repository to remote cloud services using Stencila CLI

echo "🔄 Starting sync to remote..."

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

# Push to remote using Stencila CLI
echo "⬆️  Pushing to remote..."
stencila push "${STENCILA_SYNC_FILE_PATH}" "${STENCILA_SYNC_REMOTE_URL}"

echo "✨ Sync to remote completed successfully!"
exit 0
