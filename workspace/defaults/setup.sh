#!/bin/bash

# Exit on error, undefined variables, and pipe failures
set -euo pipefail

# Empty line to separate output from line executing this script
# in the terminal
echo

# Climb up from the subdirectory to the repository root
# Remove the empty subdirectories as we go so that the repo directory is
# empty for the clone.
if [[ -n "${REPO_SUBDIR:-}" ]]; then
    # Split REPO_SUBDIR on "/" into an array
    IFS='/' read -ra SUBDIR_PARTS <<< "$REPO_SUBDIR"
    # Iterate in reverse order so that we remove the deepest directory first
    for (( idx=${#SUBDIR_PARTS[@]}-1; idx>=0; idx-- )); do
        cd .. || exit
        rmdir "${SUBDIR_PARTS[idx]}" 2>/dev/null || true
    done
fi

# Log in to GitHub if a token is provided
# It is important to set up `gh` as the Git authentication provider for `git`
# commands below
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    # Check if gh command exists
    if ! command -v gh &> /dev/null; then
        echo "❌ Error: GitHub CLI (gh) is not installed"
        exit 1
    fi
    
    echo "👤 Authenticating with GitHub"
    # Store token temporarily and clear from environment
    local_token="$GITHUB_TOKEN"
    unset GITHUB_TOKEN
    
    # Authenticate with GitHub
    if ! gh auth login --git-protocol=https --with-token <<< "$local_token"; then
        echo "❌ Error: Failed to authenticate with GitHub"
        exit 1
    fi
    
    if ! gh auth setup-git; then
        echo "❌ Error: Failed to setup git authentication"
        exit 1
    fi
    
    echo
fi

# Clone the repository if specified
if [[ -n "${GITHUB_REPO:-}" ]]; then
    # Validate repository format (owner/repo)
    if [[ ! "$GITHUB_REPO" =~ ^[a-zA-Z0-9._-]+/[a-zA-Z0-9._-]+$ ]]; then
        echo "❌ Error: Invalid repository format. Expected 'owner/repo', got '$GITHUB_REPO'"
        exit 1
    fi
    
    # Clone the repo as quickly as possible but with some commit history
    echo "📋 Cloning repository $GITHUB_REPO"
    if ! git clone "https://github.com/$GITHUB_REPO" . --depth=10 --filter=blob:none --no-checkout; then
        echo "❌ Error: Failed to clone repository $GITHUB_REPO"
        exit 1
    fi
    echo

    # Checkout the desired ref (or the default branch if not specified)
    if [[ -n "${REPO_REF:-}" ]]; then
        # First, determine and checkout the default branch
        echo "📥 Checking out default branch"
        if ! default_branch=$(basename "$(git symbolic-ref refs/remotes/origin/HEAD)"); then
            echo "❌ Error: Failed to determine default branch"
            exit 1
        fi
        if ! git checkout "$default_branch"; then
            echo "❌ Error: Failed to checkout default branch $default_branch"
            exit 1
        fi
        echo

        # Try to fetch REPO_REF from origin
        echo "📥 Attempting to fetch $REPO_REF"
        if git fetch --depth=1 origin "$REPO_REF" 2>/dev/null; then
            # Remote ref exists, check it out
            echo "✅ Remote ref found, checking out $REPO_REF"
            # Try to create a tracked branch (works for remote branches)
            if git checkout -B "$REPO_REF" --track "origin/$REPO_REF" 2>/dev/null; then
                echo "✅ Branch $REPO_REF checked out with tracking"
            else
                # Fallback for tags and commit SHAs (detached HEAD is acceptable)
                if ! git checkout FETCH_HEAD; then
                    echo "❌ Error: Failed to checkout $REPO_REF"
                    exit 1
                fi
                echo "✅ Checked out $REPO_REF (detached HEAD - tag or commit)"
            fi
        else
            # Remote branch doesn't exist, create a new local branch
            echo "✅ Remote ref not found, creating new branch $REPO_REF"
            if ! git checkout -b "$REPO_REF"; then
                echo "❌ Error: Failed to create branch $REPO_REF"
                exit 1
            fi
            # Set up tracking to origin/REPO_REF
            if ! git branch --set-upstream-to="origin/$REPO_REF" 2>/dev/null; then
                echo "📋 Note: Could not set up tracking for branch; will be set when pushed"
            fi
        fi
        echo
    else
        # Derive the default branch name from the remote HEAD reference
        echo "📥 Checking out default branch"
        if ! default_branch=$(basename "$(git symbolic-ref refs/remotes/origin/HEAD)"); then
            echo "❌ Error: Failed to determine default branch"
            exit 1
        fi
        if ! git checkout "$default_branch"; then
            echo "❌ Error: Failed to checkout default branch $default_branch"
            exit 1
        fi
        echo
    fi
fi

# Change back down to the subdirectory
# Use mkdir just in case the clone failed or the subdir was not actually
# in the repo
if [[ -n "${REPO_SUBDIR:-}" ]]; then
    mkdir -p "$REPO_SUBDIR"
    cd "$REPO_SUBDIR" || exit
fi

# Check if there are any Python or R dependencies
PYTHON_DEPS=false
R_DEPS=false

if [[ -f "pyproject.toml" || -f "requirements.txt" ]]; then
    PYTHON_DEPS=true
fi

if [[ -f "renv.lock" || -f "DESCRIPTION" ]]; then
    R_DEPS=true
fi

# If no R or Python dependencies, then copy over default Python dependencies
if [[ "$PYTHON_DEPS" = false && "$R_DEPS" = false ]]; then
    echo "🐍 No language dependencies detected, creating default Python environment"
    if ! cp /home/workspace/stencila/defaults/pyproject.toml ./; then
        echo "❌ Error: Failed to copy default pyproject.toml"
        exit 1
    fi
    # Also copy uv.lock if it exists for reproducible builds
    if [[ -f /home/workspace/stencila/defaults/uv.lock ]]; then
        if ! cp /home/workspace/stencila/defaults/uv.lock ./; then
            echo "⚠️  Warning: Failed to copy default uv.lock"
        fi
    fi
    echo
fi

# Trust mise config files if they exist (workspace container only)
if [[ -f "mise.toml" ]] || [[ -f ".mise.toml" ]] || [[ -f "mise.local.toml" ]] || [[ -f ".mise.local.toml" ]]; then
    echo "🔧 Trusting mise configuration"
    if ! mise trust; then
        echo "⚠️  Warning: Failed to trust mise config, installation may fail"
    fi
    echo
fi

echo "🎉 Setup complete!"
echo
echo "🗑️ You can close this terminal window if you wish."
