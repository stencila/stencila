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

if [[ -f "renv.lock" ]]; then
    echo "📦 Installing dependencies from renv.lock"
    Rscript -e "invisible(renv::restore())"
    R_DEPS=true
elif [[ -f "DESCRIPTION" ]]; then
    echo "📦 Installing dependencies from DESCRIPTION file"
    Rscript -e "invisible(renv::install())"
    R_DEPS=true
fi

if [[ -z "$PYTHON_DEPS" && -z "$R_DEPS" ]]; then
    echo "📦 Installing Python packages in default pyproject.toml"
    cp .stencila/workspace/pyproject.toml ./
    uv venv && uv sync
fi

echo "🎉 Setup complete!"
