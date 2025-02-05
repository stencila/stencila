#!/bin/bash

echo "🏗️ Setting up workspace"

if [ -f "pyproject.toml" ]; then
    echo "📦 Installing dependencies from pyproject.toml..."
    uv venv && uv pip install -e .
elif [ -f "requirements.txt" ]; then
    echo "📦 Installing dependencies from requirements.txt..."
    uv venv && uv pip install -r requirements.txt
fi

echo "🎉 Setup complete!"
