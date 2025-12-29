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

        # If REPO_RESET_FROM_DEFAULT is set, create/reset REPO_REF from the default branch
        # This is used by on-init.sh and on-schedule.sh to ensure a clean branch from default
        if [[ "${REPO_RESET_FROM_DEFAULT:-}" == "true" ]]; then
            echo "🌿 Creating/resetting branch $REPO_REF from $default_branch"
            if ! git checkout -B "$REPO_REF"; then
                echo "❌ Error: Failed to create branch $REPO_REF"
                exit 1
            fi
            echo "✅ Branch $REPO_REF created from $default_branch"

            # Verify we're on the expected branch before force-pushing
            CURRENT_BRANCH=$(git branch --show-current)
            if [[ "$CURRENT_BRANCH" != "$REPO_REF" ]]; then
                echo "❌ Error: Expected to be on branch '$REPO_REF' but on '$CURRENT_BRANCH'"
                exit 1
            fi

            # Push branch to remote (force to handle reset from default branch)
            echo "⬆️ Pushing branch to remote repository..."
            git push -u origin HEAD --force
            echo "✅ Branch $REPO_REF pushed to remote"
            echo
        else
            # Try to fetch REPO_REF from origin
            echo "📥 Attempting to fetch $REPO_REF"

            # BRANCH CHECKOUT STRATEGY:
            # 1. For remote branches: Fetch with explicit refspec to create the remote
            #    tracking ref, then checkout WITHOUT --track (which requires git to
            #    recognize the ref as a "branch"), then set up tracking manually.
            #    This avoids "cannot set up tracking information; starting point is not
            #    a branch" errors in fresh containers.
            # 2. For commits/tags: Try direct checkout first (if in shallow history),
            #    then fetch and checkout FETCH_HEAD (detached HEAD is correct).
            # 3. For new branches: Create local branch only after confirming it's not
            #    a typo'd commit SHA (see hex pattern check below).

            # First, try to fetch directly to the remote tracking branch
            # This explicitly creates refs/remotes/origin/$REPO_REF
            if git fetch --depth=1 origin "$REPO_REF:refs/remotes/origin/$REPO_REF" 2>/dev/null; then
                echo "✅ Remote ref found, checking out $REPO_REF"

                # Create local branch from the remote ref (without --track)
                if ! git checkout -B "$REPO_REF" "origin/$REPO_REF"; then
                    echo "❌ Error: Failed to checkout $REPO_REF"
                    exit 1
                fi

                # Set up tracking manually after branch exists
                if git branch --set-upstream-to="origin/$REPO_REF" "$REPO_REF" 2>/dev/null; then
                    echo "✅ Branch $REPO_REF checked out with tracking"
                else
                    echo "✅ Branch $REPO_REF checked out"
                fi
            else
                # Explicit refspec fetch failed, might be a tag, commit SHA, or new branch

                # Try to checkout directly (works for commits already in history)
                if git checkout "$REPO_REF" 2>/dev/null; then
                    echo "✅ Checked out $REPO_REF (detached HEAD - commit)"
                # Try fetching as a tag or full commit SHA
                elif git fetch --depth=1 origin "$REPO_REF" 2>/dev/null; then
                    echo "✅ Remote ref found, checking out $REPO_REF"
                    if ! git checkout FETCH_HEAD; then
                        echo "❌ Error: Failed to checkout $REPO_REF"
                        exit 1
                    fi
                    echo "✅ Checked out $REPO_REF (detached HEAD - tag or commit)"

                # Check if it looks like a commit SHA (7-64 hex characters)
                # This pattern matches:
                #   - Git short SHAs (7-39 chars)
                #   - Full SHA-1 hashes (40 chars)
                #   - Full SHA-256 hashes (64 chars)
                #
                # LIMITATION: This will also match legitimate branch names that are:
                #   - 7-64 characters long
                #   - Contain only hex characters [0-9a-fA-F]
                #
                # Examples of branch names that will be blocked:
                #   - deadbeef (8 chars, all hex)
                #   - 20250115 (8 chars, all hex - date-based release branch)
                #   - cafebabe (8 chars, all hex)
                #   - Any 40-char hex string (treated as SHA-1)
                #   - Any 64-char hex string (treated as SHA-256)
                #
                # WORKAROUND: Use non-hex characters in branch names:
                #   - release-20250115 instead of 20250115
                #   - v-deadbeef instead of deadbeef
                #   - Or create such branches manually first
                #
                # RATIONALE: Prevents typos in commit SHAs from silently creating
                # spurious branches. The trade-off favors catching SHA errors over
                # allowing uncommon hex-only branch names.
                elif echo "$REPO_REF" | grep -qE '^[0-9a-fA-F]{7,64}$'; then
                    echo "❌ Error: Commit $REPO_REF not found in repository"
                    exit 1
                else
                    # Doesn't look like a commit, create a new local branch
                    echo "⚠️  Remote ref not found, creating new branch $REPO_REF"
                    if ! git checkout -b "$REPO_REF"; then
                        echo "❌ Error: Failed to create branch $REPO_REF"
                        exit 1
                    fi
                    # Set up tracking to origin/REPO_REF
                    if ! git branch --set-upstream-to="origin/$REPO_REF" 2>/dev/null; then
                        echo "📋 Note: Could not set up tracking for branch; will be set when pushed"
                    fi
                fi
            fi
            echo
        fi
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

    # Configure git identity if not already configured
    if [[ -z "$(git config --get user.name || true)" ]]; then
        git config user.name "${GIT_AUTHOR_NAME:-Stencila}"
    fi
    if [[ -z "$(git config --get user.email || true)" ]]; then
        git config user.email "${GIT_AUTHOR_EMAIL:-noreply@stencila.io}"
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
