all: fix test audit build

# Install dependencies
install:
ifdef CI
	cd .. && npm ci
else
	cd .. && npm install
endif

# Make formatting and linting fixes
fix: install
	npm run fix

# Run linting checks
lint: install
	npm run lint

# Run tests
# Currently this just checks that the TypeScript compiles
# since there is not really any code here beyond type definitions
test: install
	npm test

# List outdated dependencies
outdated: install
	npm outdated

# Audit dependencies
audit: install
	npm audit fix

# Build the package
build: install
	npm run build

# Check the package before releasing
check: install
	npm run check

# Publish the package
publish: build
	npm publish --access public

# Clean up development artifacts
clean:
	rm -rf ./dist
