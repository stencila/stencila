all: setup build docs

# Setup the local development environment
setup:
	npm install

# Check schema is valid
test:
	npm test

# Build
build:
	npm run build

# Generate documentation
docs:
	npm run docs

# Clean up local development environment
clean:
	npm run clean
