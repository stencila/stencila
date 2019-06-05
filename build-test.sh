#!/usr/bin/env bash

# A script for testing that the built binary works

set -e

./bin/stencila --help

./bin/stencila setup
