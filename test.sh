#!/usr/bin/env bash

# A script for testing that the built binary can at least
# be run without errors

set -e

./bin/stencila setup
./bin/stencila --help
