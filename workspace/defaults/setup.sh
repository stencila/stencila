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
        rmdir "${SUBDIR_PARTS[idx]}"
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
        echo "📥 Checking out $REPO_REF"
        if ! git fetch --depth=1 origin "$REPO_REF"; then
            echo "❌ Error: Failed to fetch $REPO_REF"
            exit 1
        fi
        if ! git checkout FETCH_HEAD; then
            echo "❌ Error: Failed to checkout $REPO_REF"
            exit 1
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

# Install tools from mise.toml if present
if [[ -f "mise.toml" ]]; then
    echo "🔧 Installing tools from mise.toml"
    if ! command -v mise &> /dev/null; then
        echo "❌ Error: mise is not installed"
        exit 1
    fi
    if ! mise install; then
        echo "❌ Error: Failed to install tools from mise.toml"
        exit 1
    fi
    echo
fi

# Install any Python dependencies
if [[ -f "pyproject.toml" ]]; then
    echo "🐍 Installing dependencies from pyproject.toml"
    if ! (uv venv && uv sync); then
        echo "❌ Error: Failed to install Python dependencies from pyproject.toml"
        exit 1
    fi
    echo
    PYTHON_DEPS=true
elif [[ -f "requirements.txt" ]]; then
    echo "🐍 Installing dependencies from requirements.txt"
    if ! (uv venv && uv pip install -r requirements.txt); then
        echo "❌ Error: Failed to install Python dependencies from requirements.txt"
        exit 1
    fi
    echo
    PYTHON_DEPS=true
fi

# Install any R dependencies
if [[ -f "renv.lock" ]]; then
    echo "📦 Installing dependencies from renv.lock"
    if ! Rscript -e "invisible(renv::restore())"; then
        echo "❌ Error: Failed to install R dependencies from renv.lock"
        exit 1
    fi
    echo
    R_DEPS=true
elif [[ -f "DESCRIPTION" ]]; then
    echo "📦 Installing dependencies from DESCRIPTION file"
    if ! Rscript -e "invisible(renv::install())"; then
        echo "❌ Error: Failed to install R dependencies from DESCRIPTION file"
        exit 1
    fi
    echo
    R_DEPS=true
fi

# If no R or Python dependencies, then install default Python dependencies
if [[ -z "${PYTHON_DEPS:-}" && -z "${R_DEPS:-}" ]]; then
    echo "🐍 Installing Python packages in default pyproject.toml"
    if ! cp /home/workspace/stencila/defaults/pyproject.toml ./; then
        echo "❌ Error: Failed to copy default pyproject.toml"
        exit 1
    fi
    if ! (uv venv && uv sync); then
        echo "❌ Error: Failed to install default Python dependencies"
        exit 1
    fi
    echo
fi

# Setup a `.stencila` folder so that tracked files are visible to the user
if ! mkdir -p .stencila; then
    echo "❌ Error: Failed to create .stencila directory"
    exit 1
fi

echo "🎉 Setup complete!"
echo
echo "🗑️ You can close this terminal window if you wish."
