#!/usr/bin/env bash

# Integration tests

# Record errors
errors=0
error_handler () {
  (( errors++ ))
}
trap error_handler ERR


# Run a Stencila Docker container which provides all external language contexts
docker run --detach --publish 2100:2000 stencila/alpha
sleep 5

# Configured using Docker container as only peer
STENCILA_PEERS=http://localhost:2100 node tools/runner.js tests/documents/external-language-cells.html

# Configured using peer dicovery
STENCILA_DISCOVER=30 node tools/runner.js tests/documents/external-language-cells.html


# Exit with number of errors
exit $errors
