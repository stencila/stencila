#!/bin/bash

set -e

export PYTHONPATH=${PYTHONPATH}:${PWD}

python3 tests/article.py

mypy tests/article.py
