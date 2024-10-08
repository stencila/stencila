all: fix test build

# Install dependencies
# Including making local modules that are dependencies
# Note: this package is not currently part of the repo's npm
# workspace and so, unlike other packages in this repo, the
# install is done in this folder.
install:
ifdef CI
	npm ci
else
	npm install
endif
	make -C ../web build

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
	npm run compile

# Build the extension including installing the CLI
build: install
	npm run build
	bash install-cli.sh

# Install the extension locally
install-ext: compile
	npm run package
	code --install-extension stencila-*.vsix

# Clean up development artifacts
clean:
	rm -rf cli dist out
