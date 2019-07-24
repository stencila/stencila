# What? A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`.
#
# Why? Because Makefiles provide a common entry point for developers
# that are independent of the language and tooling used. In just
# about all our repos you can run `make setup` to get a dev setup
# and `cat Makefile` to quickly understand what to do next. Regardless
# or whether it's a Typescript, Python or R project.

all: setup lint test build docs


setup: setup-ts setup-py setup-r

setup-ts:
	npm install

setup-py:
	pip3 install --user --upgrade -r requirements-dev.txt

setup-r:
	Rscript -e "install.packages('devtools')"
	Rscript -e "devtools::install_github(c('jimhester/lintr', 'klutometis/roxygen', 'r-lib/covr', 'r-lib/testthat'))"


lint: lint-ts lint-py lint-r

lint-ts:
	npm run lint

lint-py:
	pylint python
	mypy python

lint-r:
	Rscript -e 'lintr::lint_package()'


test: test-ts test-py test-r

test-ts:
	npm test

test-py:
	tox

test-r:
	Rscript -e 'devtools::test()'

test-r-cover:
	Rscript -e 'devtools::document()'
	Rscript -e 'covr::package_coverage()'


build: build-ts build-py build-r

build-ts:
	npm run build

build-py:
	python3 setup.py sdist bdist_wheel

build-r:
	R CMD build . && R CMD check *.tar.gz


.PHONY: docs
docs:
	npm run docs

docs-r:
	Rscript -e 'devtools::document()'

watch:
	npm run watch


clean: clean-ts clean-py clean-r

clean-ts:
	npm run clean

clean-py:
	rm -rf build .coverage coverage.xml *.egg-info .tox **/__pycache__

clean-r:
	rm -rf stencilaschema_*.tar.gz stencilaschema.Rcheck
