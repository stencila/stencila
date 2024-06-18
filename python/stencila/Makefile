all: fix audit test build

# Bootstrap the project
.venv:
	test -d .venv || pdm venv create --with-pip --with venv 3.10 && pdm install

# Install dependencies
pdm.lock: pyproject.toml .venv
	pdm sync
	touch pdm.lock

# Install alias for consistency
install: pdm.lock

# Make formatting and linting fixes
fix: pdm.lock
	pdm run ruff format
	pdm run ruff check --fix

# Run linting checks
lint: pdm.lock
	pdm run ruff check

# Run tests
test: pdm.lock
	pdm run maturin develop
	pdm run pytest

# Run tests with coverage
cover: pdm.lock
	pdm run maturin develop
	pdm run coverage run -m pytest
	pdm run coverage lcov

# Run benchmarks with local release build
bench: pdm.lock
	pdm run maturin develop --release
	pdm run pytest --benchmark-json benchmarks.json

# Build the Python module in debug mode and open a python shell to
# import it and try it out in
run: pdm.lock
	pdm run maturin develop
	pdm run python

# Audit dependencies
outdated:
	pdm update --dry-run --unconstrained

# Audit dependencies
audit: pdm.lock
	pdm run pip-audit

# Build Python wheel for distribution
# This is intended to be close the 'Build wheel' step in the `release.yml` workflow
build: pdm.lock
	pdm run maturin build --release --out dist --interpreter 3.10 3.11 3.12

clean:
	rm -rf .pytest_cache python/stencila/_stencila.*
