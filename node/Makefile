all: fix test audit build

# Install dependencies
install:
ifdef CI
	cd .. && npm ci
else
	cd .. && npm install
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

# List outdated dependencies
outdated: install
	npm outdated

# Audit dependencies
audit: install
	npm audit fix

# Build the package
# As for `test`, must ensure that @stencila/types is built
build: install
	make -C ../typescript build
	npm run build

# Check the package before releasing
check: install
	npm run check

# Clean up development artifacts
clean:
	rm -rf bindings.d.cts *.node ./dist
