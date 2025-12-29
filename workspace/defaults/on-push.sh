#!/usr/bin/env bash

# Runs on each git push when workspace watch is enabled.
# Pushes site and outputs.

set -euo pipefail

# Navigate to the repository directory
REPO_DIR="/home/workspace/${GITHUB_REPO}"
if [[ ! -d "${REPO_DIR}" ]]; then
    echo "Error: Repository directory not found: ${REPO_DIR}"
    exit 1
fi

cd "${REPO_DIR}"
echo "📁 Working directory: $(pwd)"

echo "🚀 Updating site and outputs after git push..."

stencila push --site --outputs

echo "✨ Site and outputs updated successfully!"
