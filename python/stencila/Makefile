all: fix audit test build

# Bootstrap the project with minimum dependencies (pip in required for maturin)
bootstrap:
	pdm venv create --with-pip --with venv 3.10

# Install dependencies
pdm.lock: pyproject.toml
	pdm sync
	touch pdm.lock

# Install alias for consistency
install: pdm.lock

# Make formatting and linting fixes
fix: pdm.lock
	pdm run ruff format

# Run linting checks
lint: pdm.lock
	pdm run ruff check

# Run tests
test: pdm.lock
	pdm run maturin develop --skip-install
	pdm run pytest

# Run tests with coverage
cover: pdm.lock
	pdm run maturin develop --skip-install
	pdm run coverage run -m pytest
	pdm run coverage lcov

# Run benchmarks with local release build
bench: pdm.lock
	pdm run maturin develop --release
	pdm run pytest --benchmark-json benchmarks.json

# Build the Python module in debug mode and open a python shell to
# import it and try it out in
run: pdm.lock
	pdm run maturin develop --skip-install
	pdm run python

# List outdated dependencies
outdated:
	pdm show --outdated
	cargo outdated -d 1

# Audit dependencies
audit: pdm.lock
	pdm run pip-audit

# Build Python wheel for distribution
# This is intended to be close the 'Build wheel' step in the `release.yml` workflow
build: pdm.lock
	pdm run maturin build --release --out dist --interpreter 3.10 3.11 3.12

clean:
	rm -rf .pytest_cache python/stencila/_stencila.*
