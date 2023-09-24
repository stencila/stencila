all: fix test audit build

# Install dependencies
node_modules: package.json package-lock.json
ifdef CI
	npm ci
else
	npm install
endif

# Setup local development environment
setup: node_modules

# Make formatting and linting fixes
fix: node_modules
	npm run fix

# Run linting checks
lint: node_modules
	npm run lint

# Run tests
test: node_modules
	npm test

# List outdated dependencies
outdated: node_modules
	npm outdated

# Audit dependencies
audit: node_modules
	npm audit fix

# Build the package
build: node_modules
	npm run build

# Clean up development artifacts
clean:
	rm -rf ./dist ./node_modules
