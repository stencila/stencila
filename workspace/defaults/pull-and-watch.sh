#!/usr/bin/env bash

# Pulls a file from a remote service and creates a watch for it.
# Used for the "Edit on Google Docs / M365" workflow.

set -euo pipefail

# Validate required environment variables
if [[ -z "${STENCILA_PULL_FILE_PATH:-}" ]]; then
    echo "❌ Error: STENCILA_PULL_FILE_PATH environment variable is required"
    exit 1
fi

if [[ -z "${STENCILA_PULL_REMOTE_URL:-}" ]]; then
    echo "❌ Error: STENCILA_PULL_REMOTE_URL environment variable is required"
    exit 1
fi

echo "⬇️  Pulling ${STENCILA_PULL_FILE_PATH} from ${STENCILA_PULL_REMOTE_URL} ..."

if [[ -n "${STENCILA_WATCH_DIRECTION:-}" ]]; then
    stencila pull "${STENCILA_PULL_FILE_PATH}" --from "${STENCILA_PULL_REMOTE_URL}" --watch --direction "${STENCILA_WATCH_DIRECTION}"
    echo "✨ Pull and watch completed successfully!"
else
    stencila pull "${STENCILA_PULL_FILE_PATH}" --from "${STENCILA_PULL_REMOTE_URL}"
    echo "✨ Pull completed successfully!"
fi
