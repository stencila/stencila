all: fix test build

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

# Run linting checks
test: install
	npm test

# Compile (build in development mode)
compile: install
	npm run compile

# Clean up development artifacts
clean:
	rm -rf dist out
