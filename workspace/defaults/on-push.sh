#!/usr/bin/env bash

# Runs on each git push when workspace watch is enabled.
# Pushes site and outputs.

set -euo pipefail

echo "🚀 Updating site and outputs after git push..."

stencila push --site --outputs

echo "✨ Site and outputs updated successfully!"
