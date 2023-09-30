all: fix audit test build

# Setup local development environment
setup:
	poetry install

# Make formatting and linting fixes
fix: setup
	poetry run black python --exclude types

# Run linting checks
lint: setup
	poetry run black python --exclude types --check --diff

# Run tests
test:
	poetry run maturin develop
	poetry run pytest

# Build the Python module in debug mode and open a python shell to
# import it and try it out in
run:
	poetry run maturin develop
	poetry run python

# List outdated dependencies
outdated:
	poetry show --outdated
	cargo outdated -d 1

# Build Python wheel for distribution
build:
	poetry run maturin build
