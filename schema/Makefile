# A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`

all: format lint test build docs

setup:
	npm install --force

format:
	npm run format

lint:
	npm run lint

test:
	npm test

build:
	npm run build

docs:
	npm run docs
.PHONY: docs

clean:
	npm run clean
