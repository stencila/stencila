all: fix test audit build

# Install dependencies
#
# This does not presently include Rust (since that is uneccessary),
# but ensures install of the top level NPM workspaces,
# and Python module.
install:
	make -C typescript -B install
	make -C node -B install
	make -C python -B install

# Make formatting and linting fixes
fix:
	make -C rust fix
	make -C typescript fix
	make -C node fix
	make -C python fix

# Run linting checks
lint:
	make -C rust lint
	make -C typescript lint
	make -C node lint
	make -C python lint

# Run tests
test:
	make -C rust test
	make -C typescript test
	make -C node test
	make -C python test

# Run tests with coverage
cover:
	make -C rust cover

# Run checks (e.g. of packaging)
check:
	make -C typescript check

# List outdated dependencies
outdated:
	make -C rust outdated
	make -C typescript outdated
	make -C node outdated
	make -C python outdated

# Run accessibility checks
a11y:
	make -C rust a11y

# Audit dependencies
audit:
	make -C rust audit
	make -C typescript audit
	make -C node audit
	make -C python audit

# Build packages
build:
	make -C rust build
	make -C typescript build
	make -C node build
	make -C python build

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

# Clean up development artifacts
clean:
	make -C rust clean
	make -C typescript clean
	make -C node clean
	make -C python clean
