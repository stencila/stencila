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
# It's necessary to build the @stencila/types package first because
# at present that is linked in the package.json with a relative path
test: package-lock.json
	make -C ../typescript build
	npm run build:debug
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

# Clean up development artifacts
clean:
	rm -rf bindings.* *.node ./dist
