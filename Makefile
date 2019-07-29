# What? A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`.
#
# Why? Because Makefiles provide a common entry point for developers
# that are independent of the language and tooling used. In just
# about all our repos you can run `make setup` to get a dev setup
# and `cat Makefile` to quickly understand what to do next. Regardless
# or whether it's a Typescript, Python or R project.

all: setup lint test build docs


setup: setup-ts setup-py

setup-ts:
	npm install

setup-py:
	pip3 install --user --upgrade -r py/requirements-dev.txt


lint: lint-ts lint-py

lint-ts:
	npm run lint

lint-py:
	pylint py/stencila/schema
	mypy py/stencila/schema


test: test-ts test-py

test-ts:
	npm test

test-py:
	cd py && tox


build: build-ts build-py

build-ts:
	npm run build

build-py:
	cd py && python3 setup.py sdist bdist_wheel


.PHONY: docs
docs:
	npm run docs


watch:
	npm run watch


clean: clean-ts clean-py

clean-ts:
	npm run clean

clean-py:
	rm -rf py/build py/.coverage py/coverage.xml py/*.egg-info py/.tox **/__pycache__
