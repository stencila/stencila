# A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`, or the
# `Makefiles` for each language folder e.g. `python/Makefile`.

all: format lint test build docs

setup:
	npm install
	make -C python setup
	make -C r setup

format:
	npm run format

lint:
	npm run lint
	make -C python lint
	make -C r lint

test:
	npm test
	make -C python test
	make -C r test

build:
	npm run build
	make -C python build
	make -C r build

docs:
	npm run docs
	make -C python docs
	make -C r docs
.PHONY: docs

clean:
	npm run clean
	make -C python clean
	make -C r clean

# Build Docker image for development
build-image:
	docker build \
	  --build-arg USER_ID=$$(id -u) \
    --build-arg GROUP_ID=$$(id -g) \
		-t stencila/schema .

# Run an interactive shell in Docker container
run-image:
	docker run --rm -it -v $$PWD:/code -w /code stencila/schema bash
