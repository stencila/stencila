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
	npm run test

# Run watch
run: install
	npm start

# List outdated dependencies
outdated: install
	npm outdated

# Audit dependencies
audit: install
	npm run unused
	cd .. && npm run audit

# Build the package
build: install clean
	npm run build

# Clean up development artifacts
clean:
	mv dist/.gitignore dist.gitignore
	rm -rf dist
	mkdir dist
	mv dist.gitignore dist/.gitignore
