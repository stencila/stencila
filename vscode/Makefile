all: fix test build

# Install dependencies
# Note: this package is not currently part of the repo's npm
# workspace and so, unlike other packages in this repo, the
# install is done in this folder.
install:
ifdef CI
	npm ci
else
	npm install
endif

# Make formatting and linting fixes
fix: install
	npm run fix

# Run linting checks
lint: install
	npm run lint

# Run linting checks
test: install
	npm test

# Compile (build in development mode)
compile: install
	make -C ../ts build
	npm run compile

# Package into a vsix file for current platform
package: install
	npm run package

# Clean up development artifacts
clean:
	rm -rf dist out
