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
test: install
	npm test

# List outdated dependencies
outdated: install
	npm outdated

# Audit dependencies
audit: install
	npm run unused
	cd .. && npm run audit

# Build the package
build: install
	npm run build

# Run package publishing checks
pubcheck: build
	npm run pubcheck

# Publish the package
publish: build
	npm publish --access public

# Clean up development artifacts
clean:
	rm -rf ./dist
