all: setup build test

.PHONY: setup build test

# Install requirements
setup:
	npm install

# Build extension
build:
	@mkdir -p build
	node wrap.js
	node-gyp configure build

# Run tests
test:
	npm test

# Run tests with coverage
cover:
	npm run cover
