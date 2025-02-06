#!/bin/bash

echo "🏗️ Setting up workspace"

if [[ -f "pyproject.toml" ]]; then
    echo "📦 Installing dependencies from pyproject.toml"
    uv venv && uv sync
    PYTHON_DEPS=true
elif [[ -f "requirements.txt" ]]; then
    echo "📦 Installing dependencies from requirements.txt"
    uv venv && uv pip install -r requirements.txt
    PYTHON_DEPS=true
fi

if [[ -z "$PYTHON_DEPS" ]]; then
    echo "📦 Using and installing default pyproject.toml"
    cp .stencila/workspace/pyproject.toml .
    uv venv && uv sync
fi

echo "🎉 Setup complete!"
