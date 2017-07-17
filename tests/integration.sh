#!/usr/bin/env bash

# Integration tests

# Record errors
errors=0
error_handler () {
  (( errors++ ))
}
trap error_handler ERR


# Run a Stencila Docker container which provides a Node Host (as have in Desktop)
docker run --detach --publish 2000:2000 stencila/iota
# Run a Stencila Docker container which provides several language Hosts
docker run --detach --publish 2100:2000 stencila/alpha

sleep 5

# Configured using one of the containers as only peer
STENCILA_PEERS=http://localhost:2100 node tools/runner.js tests/documents/external-language-cells.html

# Configured using peer dicovery (this is current configuration for Desktop)
STENCILA_DISCOVER=30 node tools/runner.js tests/documents/external-language-cells.html


# Exit with number of errors
exit $errors
