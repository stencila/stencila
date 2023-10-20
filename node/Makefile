all: fix test audit build

# Install dependencies
install:
ifdef CI
	cd .. && npm ci --ignore-scripts
else
	cd .. && npm install --ignore-scripts
endif

# Make formatting and linting fixes
fix: install
	npm run fix

# Run linting checks
lint: install
	npm run lint

# Run tests
# It's necessary to build the @stencila/types package first because
# at present that is linked in the package.json with a relative path
test: install
	make -C ../typescript build
	npm run build:debug
	npm test

# Run tests with coverage
cover: install
	make -C ../typescript build
	npm run build:debug
	npm run cover

# List outdated dependencies
outdated: install
	npm outdated

# Audit dependencies
audit: install
	npm run unused
	cd .. && npm run audit

# Build the Node binary add-on only
addon: install
	npm run build:addon

# Build the TypeScript into `dist` folder
# It is necessary to build the addon (just debug for speed) because that
# generates the `bindings.d.cts` file.
dist: install
	make -C ../typescript build
	npm run build:debug
	npm run build:dist
.PHONY: dist

# Run package publishing checks
pubcheck: install
	npm run pubcheck

# Publish the package
publish: dist
	npm publish --access public

# Clean up development artifacts
clean:
	rm -rf bindings.d.cts *.node ./dist
