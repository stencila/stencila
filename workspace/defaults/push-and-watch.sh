#!/usr/bin/env bash

# Pushes a file to a remote service and creates a watch for it.
# Used for the "Edit on Google Docs / M365" workflow.

set -euo pipefail

# Validate required environment variables
if [[ -z "${STENCILA_PUSH_FILE_PATH:-}" ]]; then
    echo "❌ Error: STENCILA_PUSH_FILE_PATH environment variable is required"
    exit 1
fi

if [[ -z "${STENCILA_PUSH_REMOTE_SERVICE:-}" ]]; then
    echo "❌ Error: STENCILA_PUSH_REMOTE_SERVICE environment variable is required"
    exit 1
fi

if [[ -z "${STENCILA_WATCH_DIRECTION:-}" ]]; then
    echo "❌ Error: STENCILA_WATCH_DIRECTION environment variable is required"
    exit 1
fi

echo "⬆️  Pushing ${STENCILA_PUSH_FILE_PATH} to ${STENCILA_PUSH_REMOTE_SERVICE} ..."

stencila push "${STENCILA_PUSH_FILE_PATH}" --to "${STENCILA_PUSH_REMOTE_SERVICE}" --watch --direction "${STENCILA_WATCH_DIRECTION}"

echo "✨ Push and watch completed successfully!"
