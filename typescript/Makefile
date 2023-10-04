all: fix test audit build

# Install dependencies
package-lock.json: package.json
ifdef CI
	cd .. && npm ci
else
	cd .. && npm install
endif
	touch package-lock.json

# Install alias for consistency
install: package-lock.json

# Make formatting and linting fixes
fix: package-lock.json
	npm run fix

# Run linting checks
lint: package-lock.json
	npm run lint

# Run tests
# Currently this just checks that the TypeScript compiles
# since there is not really any code here beyond type definitions
test: package-lock.json
	npm test

# List outdated dependencies
outdated: package-lock.json
	npm outdated

# Audit dependencies
audit: package-lock.json
	npm audit fix

# Build the package
build: package-lock.json
	npm run build

# Check the package before releasing
check: package-lock.json
	npm run check

# Release the package
release: build
	npm publish --access public

# Clean up development artifacts
clean:
	rm -rf ./dist
