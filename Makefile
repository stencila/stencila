all: fix test audit build

# Install dependencies
setup:
	make -C rust setup
	make -C typescript setup
	make -C python setup

# Make formatting and linting fixes
fix:
	make -C rust fix
	make -C typescript fix
	make -C python fix

# Run linting checks
lint:
	make -C rust lint
	make -C typescript lint
	make -C python lint

# Run tests
test:
	make -C rust test
	make -C typescript test

# List outdated dependencies
outdated:
	make -C rust outdated
	make -C typescript outdated
	make -C python outdated

# Audit dependencies
audit:
	make -C rust audit
	make -C typescript audit

# Build packages
build:
	make -C rust build
	make -C typescript build

# Build Docker image
docker:
	docker build --tag stencila/stencila .

# Generate generated source and docs
generated:
	make -C rust generated

# Generate examples in alternative formats
examples:
	make -C rust examples
.PHONY: examples

# Cut a release
release:
	cargo release -p stencila --tag-prefix '' --no-publish --execute alpha

# Clean up development artifacts
clean:
	make -C rust clean
	make -C typescript clean
