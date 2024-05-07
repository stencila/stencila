all: fix test audit build

# Install dependencies
install:
ifdef CI
	cd ../.. && npm ci --ignore-scripts
else
	cd ../.. && npm install --ignore-scripts
endif
	make -C ../../ts build

# Make formatting and linting fixes
fix: install
	npm run fix

# Run linting checks
lint: install
	npm run lint

# Run tests
test: install
	npm run build:debug
	npm test

# Run tests with coverage
cover: install
	npm run build:debug
	npm run cover

# Run benchmarks with release build
# This moves the release build of the addon to `stencila.node`
# which is the expected path of the addon.
bench: install
	npm run build:addon
	mv stencila.*.node stencila.node
	npm run bench
.PHONY: bench

# List outdated dependencies
outdated: install
	npm outdated

# Audit dependencies
audit: install
	npm run unused
	cd ../.. && npm run audit

# Build the Node binary add-on only
addon: install
	npm run build:addon

# Build the TypeScript into `dist` folder
# It is necessary to build the addon (just debug for speed) because that
# generates the `bindings.d.cts` file.
dist: install
	npm run build:debug
	npm run build:dist
.PHONY: dist

# Run package publishing checks
pubcheck: dist
	npm run pubcheck

# Publish the package
publish: dist
	npm publish --access public

# Clean up development artifacts
clean:
	rm -rf bindings.d.cts *.node ./dist
