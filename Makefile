all: setup lint test cover build docs

# Setup the local development environment

setup: setup-ts setup-py

setup-ts:
	npm install

setup-py:
	pip3 install --user --upgrade -r requirements-dev.txt


# Add Git hooks

hooks:
	cp pre-commit.sh .git/hooks/pre-commit


# Lint code

lint: lint-ts lint-py
	
lint-ts:
	npm run lint

lint-py: lint-py-code lint-py-types

lint-py-code:
	pylint --exit-zero src

lint-py-types:
	mypy src


# Run tests

test: test-ts test-py

test-ts:
	npm test

test-py:
	tox


# Run tests with coverage

cover: cover-ts cover-py
	
cover-ts:
	npm run cover

cover-py:
	tox -e cover


# Run any development servers

run:
	npm start


# Build packages

build: build-ts
.PHONY: build

build-ts:
	npm run build

build-py:
	echo "To do!"


# Generate documentation

docs: docs-ts docs-py

docs-ts:
	npm run docs

docs-py:
	echo "To do!"


# Clean up local development environment

clean: clean-ts clean-py

clean-ts:
	npm run clean

clean-py:
	echo "To do!"
