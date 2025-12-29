#!/usr/bin/env bash

# Syncs content from the local Git repository to remote cloud services using Stencila CLI

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

echo "⬆️  Pushing ${STENCILA_SYNC_FILE_PATH} to ${STENCILA_SYNC_REMOTE_URL} ..."

stencila push "${STENCILA_SYNC_FILE_PATH}" --to "${STENCILA_SYNC_REMOTE_URL}"

echo "✨ Sync to remote completed successfully!"
