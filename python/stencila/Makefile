all: fix audit test build

# Bootstrap the project
.venv:
	test -d .venv || uv venv && uv sync

# Install dependencies
uv.lock: pyproject.toml .venv
	uv sync
	touch uv.lock

# Install alias for consistency
install: uv.lock

# Make formatting and linting fixes
fix: uv.lock
	uv run ruff format
	uv run ruff check --fix

# Run linting checks
lint: uv.lock
	uv run ruff check

# Run tests
test: uv.lock
	uv run maturin develop
	uv run pytest

# Run tests with coverage
cover: uv.lock
	uv run maturin develop
	uv run coverage run -m pytest
	uv run coverage lcov

# Run benchmarks with local release build
bench: uv.lock
	uv run maturin develop --release
	uv run pytest --benchmark-json benchmarks.json

# Build the Python module in debug mode and open a python shell to
# import it and try it out in
run: uv.lock
	uv run maturin develop
	uv run python

# List outdated dependencies
outdated:
	uv pip list --outdated

# Audit dependencies
audit: uv.lock
	uv run pip-audit --ignore-vuln PYSEC-2022-43012

# Build Python wheel for distribution
# This is intended to be close the 'Build wheel' step in the `release.yml` workflow
build: uv.lock
	uv run maturin build --release --out dist --interpreter 3.10 3.11 3.12

clean:
	rm -rf .pytest_cache python/stencila/_stencila.*
