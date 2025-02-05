#!/bin/bash

echo "🏗️ Setting up workspace"

if [ ! -z "$GITHUB_TOKEN" ]; then
    echo "🪪 Logging into GitHub..."

    # Erase the GITHUB_TOKEN env var to avoid complaint about that
    echo "$GITHUB_TOKEN" | GITHUB_TOKEN= gh auth login --with-token
fi

if [ ! -z "$GITHUB_REPO" ]; then
    echo "📥 Cloning GitHub repository: $GITHUB_REPO"

    # Temporarily move the `.stencila` folder that this file is in
    # so that Git will not complain about it not being empty
    mv .stencila /tmp

    # Only clone using gh cli if a token is provided, otherwise fall back to
    # plain git which works without a login and assume repo is public.
    if [ ! -z "$GITHUB_TOKEN" ]; then
        GITHUB_TOKEN= gh repo clone "$GITHUB_REPO" -- --depth=1
    else
        git clone --depth=1 "https://github.com/$GITHUB_REPO" .
    fi

    # Move back the `.stencila` folder
    mv /tmp/.stencila .
fi

if [ -f "pyproject.toml" ]; then
    echo "📦 Installing dependencies from pyproject.toml..."
    uv venv && uv pip install -e .
elif [ -f "requirements.txt" ]; then
    echo "📦 Installing dependencies from requirements.txt..."
    uv venv && uv pip install -r requirements.txt
fi

echo "🎉 Setup complete!"
