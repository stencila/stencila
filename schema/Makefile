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

# Build Docker image for development
build-image:
	docker build \
	  --build-arg USER_ID=$$(id -u) \
    --build-arg GROUP_ID=$$(id -g) \
		-t stencila/schema .

# Run an interactive shell in Docker container
run-image:
	docker run --rm -it -v $$PWD:/code -w /code stencila/schema bash
