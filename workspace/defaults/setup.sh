#!/bin/bash

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
        cd ..
        rmdir "${SUBDIR_PARTS[idx]}"
    done
fi

# Log in to GitHub if a token is provided
# It is important to set up `gh` as the Git authentication provider for `git`
# commands below
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    echo "👤 Authenticating with GitHub"
    GITHUB_TOKEN= gh auth login --git-protocol=https --with-token <<< "$GITHUB_TOKEN"
    gh auth setup-git
    echo
fi

# Clone the repository if specified
if [[ -n "${GITHUB_REPO:-}" ]]; then
    # Clone the repo as quickly as possible but with some commit history
    echo "📋 Cloning repository $GITHUB_REPO"
    git clone "https://github.com/$GITHUB_REPO" . --depth=10 --filter=blob:none --no-checkout
    echo

    # Checkout the desired ref (or the default branch if not specified)
    if [[ -n "${REPO_REF:-}" ]]; then
        echo "📥 Checking out $REPO_REF"
        git fetch --depth=1 origin "$REPO_REF"
        git checkout FETCH_HEAD
        echo
    else
        # Derive the default branch name from the remote HEAD reference
        echo "📥 Checking out default branch"
        default_branch=$(basename "$(git symbolic-ref refs/remotes/origin/HEAD)")
        git checkout "$default_branch"
        echo
    fi
fi

# Change back down to the subdirectory
# Use mkdir just in case the clone failed or the subdir was not actually
# in the repo
if [[ -n "${REPO_SUBDIR:-}" ]]; then
    mkdir -p "$REPO_SUBDIR"
    cd "$REPO_SUBDIR"
fi

# Install any Python dependencies
if [[ -f "pyproject.toml" ]]; then
    echo "📦 Installing dependencies from pyproject.toml"
    uv venv && uv sync
    echo
    PYTHON_DEPS=true
elif [[ -f "requirements.txt" ]]; then
    echo "📦 Installing dependencies from requirements.txt"
    uv venv && uv pip install -r requirements.txt
    echo
    PYTHON_DEPS=true
fi

# Install any R dependencies
if [[ -f "renv.lock" ]]; then
    echo "📦 Installing dependencies from renv.lock"
    Rscript -e "invisible(renv::restore())"
    echo
    R_DEPS=true
elif [[ -f "DESCRIPTION" ]]; then
    echo "📦 Installing dependencies from DESCRIPTION file"
    Rscript -e "invisible(renv::install())"
    echo
    R_DEPS=true
fi

# If no R or Python dependencies, then install default Python dependencies
if [[ -z "$PYTHON_DEPS" && -z "$R_DEPS" ]]; then
    echo "📦 Installing Python packages in default pyproject.toml"
    cp /home/workspace/stencila/defaults/pyproject.toml ./
    uv venv && uv sync
    echo
fi

# Setup a `.stencila` folder so that tracked files are visible to the user
mkdir -p .stencila

echo "🎉 Setup complete!"
echo
echo "🗑️ You can close this terminal window if you wish."
