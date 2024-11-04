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

# Build walkthroughs and copy into extension folder for packaging
walkthroughs:
	make -C ../docs/walkthroughs clean all
	rm -rf walkthroughs
	cp -r ../docs/walkthroughs walkthroughs
PHONY: walkthroughs

# Compile (build in development mode)
compile: install walkthroughs
	npm run compile
	cargo build -p cli

# Build the extension including installing the CLI
build: install walkthroughs
	npm run compile
	bash install-cli.sh

# Install the extension locally
install-ext: compile
	npm run package
	code --install-extension stencila-*.vsix

# Clean up development artifacts
clean:
	rm -rf cli dist out
