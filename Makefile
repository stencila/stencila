# What? A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`.
#
# Why? Because Makefiles provide a common entry point for developers
# that are independent of the language and tooling used. In just
# about all our repos you can run `make setup` to get a dev setup
# and `cat Makefile` to quickly understand what to do next. Regardless
# or whether it's a Typescript, Python or R project.

all: setup lint test build docs

setup:
	npm install

lint:
	npm run lint

test:
	npm test

build:
	npm run build

.PHONY: docs
docs:
	npm run docs

watch:
	npm run watch

clean:
	npm run clean
