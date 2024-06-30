all: fix test

# Install dependencies, including making dependencies
# in the same workspaces
install:
ifdef CI
	cd ../.. && npm ci --ignore-scripts
else
	cd ../.. && npm install --ignore-scripts
endif
	make -C ../../node/stencila-plugin build
	make -C ../../ts build

# Run tests
test: install
	npm test

# Build package
build: install
	npm run build

# Clean up development artifacts
clean:
	rm -rf ./dist
