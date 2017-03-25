# A simple Makefile providing shortcuts to NPM tasks defined in package.json
# Why? Because, for developers working in multiple Stencila repos, using alternative
# languages (e.g. Javascript, R, Python), it's nice to have a consistent command line
# interface for common development tasks (e.g. `make setup`, `make run`, `make docs`)

all: setup lint cover build docs

setup:
	npm install

run:
	npm start

lint:
	npm run lint

test:
	npm test

test-browser:
	npm run test-browser

test-one:
	npm run test-one -- $(FILE)

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
	rm -rf node_modules build tmp
