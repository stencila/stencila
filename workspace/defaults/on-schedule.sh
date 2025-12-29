#!/usr/bin/env bash

# Runs when workspace schedule is enabled.
# Pushes site and outputs.

set -euo pipefail

# Validate required environment variable
if [[ -z "${GITHUB_REPO:-}" ]]; then
    echo "❌ Error: GITHUB_REPO environment variable is required"
    exit 1
fi

REPO_DIR="/home/workspace/${GITHUB_REPO}"
if [[ ! -d "${REPO_DIR}" ]]; then
    echo "❌ Error: Repository directory not found: ${REPO_DIR}"
    exit 1
fi

cd "${REPO_DIR}"
echo "📁 Working directory: $(pwd)"

echo "🚀 Scheduled update of site and outputs"

stencila push --site --outputs

echo "✨ Site and outputs updated successfully!"
