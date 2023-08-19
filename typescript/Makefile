all: fix audit build

# Install dependencies
setup:
	npm install

# Make formatting and linting fixes
fix:
	npm run fix

# Run linting checks
lint:
	npm run lint

# Run tests
# Currently this just checks that the TypeScript compiles
# since there is not really any code here beyond type definitions
test:
	npx tsc

# List outdated dependencies
outdated:
	npm outdated

# Audit dependencies
audit:
	npm audit fix

# Build the package
build:
	npm run build

# Clean up development artifacts
clean:
	rm -rf ./dist ./node_modules
