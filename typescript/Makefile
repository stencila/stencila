all: fix audit build

# Install dependencies
node_modules: package.json package-lock.json
ifdef CI
	npm ci
else
	npm install
endif

# Setup local development environment
setup: node_modules

# Make formatting and linting fixes
fix: node_modules
	npm run fix

# Run linting checks
lint: node_modules
	npm run lint

# Run tests
# Currently this just checks that the TypeScript compiles
# since there is not really any code here beyond type definitions
test: node_modules
	npx tsc

# List outdated dependencies
outdated: node_modules
	npm outdated

# Audit dependencies
audit: node_modules
	npm audit fix

# Build the package
build: node_modules
	npm run build

# Release the package
# Only updates the version number and publishes if there have
# been changes in this folder between the current tag and the one
# before it.
release:
	git diff --quiet $(shell git describe --abbrev=0 --tags $(shell git rev-list --tags --max-count=2 | tail -n 1)) $(shell git describe --abbrev=0 --tags) -- . ; \
	if [ $$? -eq 1 ]; then \
		sed -i 's/"version":.*$$/"version": "$(shell git describe --tags --abbrev=0)",/' package.json; \
		npm publish --access public; \
	fi

# Clean up development artifacts
clean:
	rm -rf ./dist ./node_modules
