#!/usr/bin/env bash

# Redirect stdout and stderr through tee so output goes to both the
# terminal and the log file.
exec > >(tee -a "entrypoint.log") 2>&1

# Log in to GitHub if a token is provided
# It is important to set up `gh` as the Git authentication provider for `git`
# commands below
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    echo "📛 Authenticating with GitHub"
    GITHUB_TOKEN= gh auth login --git-protocol=https --with-token <<< "$GITHUB_TOKEN"
    gh auth setup-git
fi

# Clone the repository if specified
if [[ -n "${GITHUB_REPO:-}" ]]; then
    # Create the destination directory (if not already created)
    mkdir -p "$GITHUB_REPO"

    # Clone the repo as quickly as possible but with some commit history
    echo "📨 Cloning $GITHUB_REPO using git"
    git clone "https://github.com/$GITHUB_REPO" "$GITHUB_REPO" --depth=10 --filter=blob:none --no-checkout

    # Checkout the desired ref (or the default branch if not specified)
    pushd "$GITHUB_REPO" >/dev/null
    if [[ -n "${REPO_REF:-}" ]]; then
        echo "📥 Checking out $REPO_REF"
        git fetch --depth=1 origin "$REPO_REF"
        git checkout FETCH_HEAD
    else
        # Derive the default branch name from the remote HEAD reference
        echo "📥 Checking out default branch"
        default_branch=$(basename "$(git symbolic-ref refs/remotes/origin/HEAD)")
        git checkout "$default_branch"
    fi
    popd >/dev/null

    # Copy the Stencila workspace files to the working directory.
    # If REPO_SUBDIR is set, the workspace should end up in "$GITHUB_REPO/$REPO_SUBDIR/.stencila/workspace".
    # If REPO_SUBDIR is not set, it will be "$GITHUB_REPO/.stencila/workspace".
    echo "➡️  Moving workspace files"
    dest_dir="$GITHUB_REPO/${REPO_SUBDIR:-}/.stencila"
    mkdir -p "$dest_dir"
    mv .stencila/workspace "$dest_dir/"
fi

# Start OpenVSCode Server
# This should be accessed with the query parameter ?folder=$GITHUB_REPO/$GITHUB_SUBDIR
${OVS_HOME}/bin/openvscode-server --host 0.0.0.0 --port 8080 --without-connection-token
