all: setup lint cover build docs

setup:
	npm install

lint:
	npm run lint

test:
	npm test

test-documents:
	npm run test-documents

test-bundle:
	npm run test-bundle

test-browser:
	npm run test-browser

cover:
	npm run cover

build:
	npm run build
.PHONY: build

docs:
	npm run docs
.PHONY: docs

docs-serve:
	npm run docs-serve

clean:
	rm -rf node_modules
