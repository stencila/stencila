all: fix audit build

# Setup local development environment
setup:
	poetry install

# Make formatting and linting fixes
fix: setup
	poetry run black stencila --exclude types

# Run linting checks
lint: setup
	poetry run black stencila --exclude types --check --diff

# List outdated dependencies
outdated:
	poetry show --outdated
