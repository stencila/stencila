all: fix audit test build

# Install dependencies
poetry.lock: pyproject.toml
	poetry install --sync
	touch poetry.lock

# Install alias for consistency
install: poetry.lock

# Make formatting and linting fixes
fix: poetry.lock
	poetry run black python --exclude types

# Run linting checks
lint: poetry.lock
	poetry run black python --exclude types --check --diff

# Run tests
test: poetry.lock
	poetry run maturin develop
	poetry run pytest

# Run tests with coverage
cover: poetry.lock
	poetry run maturin develop
	poetry run coverage run -m pytest
	poetry run coverage lcov

# Run benchmarks with local release build
bench: poetry.lock
	poetry run maturin develop --release
	poetry run pytest --benchmark-json benchmarks.json

# Build the Python module in debug mode and open a python shell to
# import it and try it out in
run: poetry.lock
	poetry run maturin develop
	poetry run python

# List outdated dependencies
outdated:
	poetry show --outdated
	cargo outdated -d 1

# Audit dependencies
audit: poetry.lock
	poetry run pip-audit

# Build Python wheel for distribution
# This is intended to be close the 'Build wheel' step in the `release.yml` workflow
build: poetry.lock
	poetry run maturin build --release --out dist --interpreter 3.8 3.9 3.10 3.11 3.12

clean:
	rm -rf .pytest_cache python/stencila/_stencila.*
