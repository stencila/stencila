#!/usr/bin/env bash

# Runs when workspace schedule is enabled.
# Pushes site and outputs.

set -euo pipefail

echo "🚀 Running scheduled update of site and outputs..."

stencila push --site --outputs

echo "✨ Site and outputs updated successfully!"
