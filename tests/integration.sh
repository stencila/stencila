#!/usr/bin/env sh

# Integration tests

# Run a Stencila Docker container which provides all external language contexts
docker run --detach --publish 2100:2000 stencila/alpha
sleep 5

# Configured using Docker container as only peer
STENCILA_PEERS=http://localhost:2100 node tools/runner.js tests/documents/external-language-cells.html

# Configured using peer dicovery
STENCILA_DISCOVER=30 node tools/runner.js tests/documents/external-language-cells.html
