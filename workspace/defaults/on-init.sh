#!/usr/bin/env bash

# Initializes a Stencila workspace in the local Git repository

set -euo pipefail

echo "🔄 Initializing Stencila workspace..."

# Validate required environment variables
if [[ -z "${GITHUB_REPO:-}" ]]; then
    echo "❌ Error: GITHUB_REPO environment variable is required"
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

# Initialize the Stencila workspace
echo "⚙️  Running stencila init..."
stencila init --yes

# Check if there are changes to commit and push
if [[ -n "$(git status --porcelain)" ]]; then
    echo "📝 Committing and pushing initialization changes..."

    # Add all changes
    git add -A

    # Create commit
    COMMIT_MSG="Initialize Stencila workspace"
    git commit -m "${COMMIT_MSG}"
    echo "✅ Changes committed: ${COMMIT_MSG}"

    # Push to remote repository
    echo "🚀 Pushing changes to remote repository..."
    git push origin HEAD
else
    echo "ℹ️  No changes to commit"
fi

echo "✨ Stencila workspace initialization completed successfully!"
