all: fix test audit build

# Install dependencies
#
# This does not presently include Rust (since that is uneccessary),
# but ensures install of the top level NPM workspaces,
# and Python module.
install:
	make -C ts -B install
	make -C node -B install
	make -C python -B install
	make -C web -B install
	make -C vscode -B install

# Make formatting and linting fixes
fix:
	make -C rust fix
	make -C ts fix
	make -C node fix
	make -C python/stencila fix
	make -C web fix
	make -C vscode fix

# Run linting checks
lint:
	make -C rust lint
	make -C ts lint
	make -C node lint
	make -C python/stencila lint
	make -C web lint
	make -C vscode lint

# Run tests
test:
	make -C rust test
	make -C ts test
	make -C node test
	make -C python/stencila test
	make -C test test

# Run tests with coverage
cover:
	make -C rust cover
	make -C ts test
	make -C node cover
	make -C python/stencila cover

# Run and collate benchmarks
bench:
	make -C rust bench
	make -C node bench
	make -C python/stencila bench
	make -C docs/develop/benchmarks update

# List outdated dependencies
outdated:
	make -C rust outdated
	make -C ts outdated
	make -C node outdated
	make -C python/stencila outdated
	make -C web outdated

# Run accessibility checks
a11y:
	make -C rust a11y

# Run package publishing checks
pubcheck:
	make -C ts pubcheck
	make -C node pubcheck

# Audit dependencies
audit:
	make -C rust audit
	make -C ts audit
	make -C node audit
	make -C python/stencila audit
	make -C web audit

# Build packages
build:
	make -C rust build
	make -C ts build
	make -C node build
	make -C python/stencila build
	make -C web build
	make -C vscode build

# Build Docker image
docker:
	docker build --tag stencila/stencila .

# Generate generated source and docs
generated:
	make -C rust generated

# Generate examples in alternative formats
examples:
	make -C examples
.PHONY: examples

# Clean up development artifacts
clean:
	make -C rust clean
	make -C ts clean
	make -C node clean
	make -C python/stencila clean
	make -C web clean
	make -C vscode clean
