all: setup lint test cover build docs

# Setup the local development environment

setup: setup-js

setup-js:
	cd js && npm install

# Add Git hooks

hooks:
	cp pre-commit.sh .git/hooks/pre-commit


# Lint code

lint: lint-js
	
lint-js:
	cd js && npm run lint

# Run tests

test: test-js

test-js:
	cd js && npm test

# Run tests with coverage

cover: cover-js
	
cover-js:
	cd js && npm run cover

# Run benchmarks

bench: bench-js

bench-js:
	cd js && npm run bench


# Run any development servers

run:
	cd js && npm start


# Build packages

build: build-js
.PHONY: build

build-js:
	cd js && npm run build

# Generate documentation

docs: docs-js

docs-js:
	cd js && npm run docs

# Clean up local development environment

clean: clean-js

clean-js:
	cd js && npm run clean
