all: setup build test

.PHONY: setup build test

# Install requirements
install:
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

# Clean up
clean:
	rm -rf build coverage