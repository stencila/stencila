all: fix audit build

# Setup local development environment
setup:
	poetry install

# Make formatting and linting fixes
fix:
	poetry run black stencila --exclude types/software_source_code.py

# Run linting checks
lint:
	poetry run black stencila --exclude types/software_source_code.py --check --diff

# List outdated dependencies
outdated:
	poetry show --outdated
