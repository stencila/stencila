all: setup build docs

# Setup the local development environment
setup:
	npm install

# Add Git hook to test before a commit
hooks:
	cp pre-commit.sh .git/hooks/pre-commit

# Check schema is valid
test:
	npm test

# Build
build:
	npm run build

# Generate documentation
docs:
	npm run docs

# Clean up local development environment
clean:
	npm run clean
