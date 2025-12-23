#!/usr/bin/env bash

set -euo pipefail

# init.sh
# Initializes a Stencila project in the local Git repository

echo "🔄 Initializing Stencila project..."

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

# Configure git identity if not already configured
if [[ -z "$(git config --get user.name || true)" ]]; then
    git config user.name "${GIT_AUTHOR_NAME:-Stencila User}"
fi
if [[ -z "$(git config --get user.email || true)" ]]; then
    git config user.email "${GIT_AUTHOR_EMAIL:-noreply@stencila.io}"
fi

# Create or reset branch for initialization
BRANCH_NAME="stencila/init"
echo "🌿 Creating branch: ${BRANCH_NAME}"
git checkout -B "${BRANCH_NAME}"

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
    git push -u origin HEAD
    echo "✅ Changes pushed to remote repository"
else
    echo "ℹ️  No changes to commit"
fi

echo "✨ Stencila workspace initialization completed successfully!"
exit 0
