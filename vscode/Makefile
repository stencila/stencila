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

# Copy walkthroughs into extension folder for packaging
walkthroughs:
	rm -rf walkthroughs
	cp -r ../docs/walkthroughs walkthroughs
.PHONY: walkthroughs

# Compile (build in development mode)
compile: install walkthroughs
	rm -rf out
	npm run compile
	cargo build -p cli

# Build and run Docker image for local testing of extension in that environment
# It copies the local debug version of the CLI binary, rather than
# downloading it like `make build` (below) does.
docker:
	rm -rf out
	npm run compile
	cp ../target/debug/stencila cli/
	npm run package
	docker build . --tag stencila-vsix-test
	echo "Run using something like: docker run -it --rm --publish 8080:3000 -eSTENCILA_API_TOKEN=... stencila-vsix-test"

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
