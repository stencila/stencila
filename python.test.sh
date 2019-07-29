#!/bin/bash

set -e

export PYTHONPATH=${PYTHONPATH}:${PWD}/py/stencila/
export MYPYPATH=${MYPYPATH}:${PWD}/py/stencila/

python3 tests/article.py

mypy tests/article.py
