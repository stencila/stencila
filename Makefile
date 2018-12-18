all: setup lint test cover build docs

# Setup the local development environment

setup: setup-js setup-py setup-r

setup-js:
	cd js && npm install

setup-py:
	pip3 install --user --upgrade -r requirements-dev.txt

setup-r:
	Rscript -e "install.packages('devtools')"
	Rscript -e "devtools::install_github(c('jimhester/lintr', 'klutometis/roxygen', 'r-lib/bench', 'r-lib/covr', 'r-lib/testthat'))"


# Add Git hooks

hooks:
	cp pre-commit.sh .git/hooks/pre-commit


# Lint code

lint: lint-js lint-py lint-r
	
lint-js:
	cd js && npm run lint

lint-py: lint-py-code lint-py-types

lint-py-code:
	cd py && pylint --exit-zero src

lint-py-types:
	cd py && mypy src

lint-r:
	cd r && Rscript -e 'lintr::lint_package()'

# Run tests

test: test-js test-py test-r

test-js:
	cd js && npm test

test-py:
	cd py && tox

test-r:
	cd r && Rscript -e 'devtools::test()'

# Run tests with coverage

cover: cover-js cover-py cover-r
	
cover-js:
	cd js && npm run cover

cover-py:
	cd py && tox -e cover

cover-r:
	cd r && Rscript -e 'covr::package_coverage()'

# Run benchmarks

bench: bench-js bench-py bench-r

bench-js:
	cd js && npm run bench

bench-py:
	cd py && tox -e bench -- tests/bench

bench-r: install-r
	cd r/tests/bench && Rscript encoders.R


# Run integration tests

integ: integ-py

integ-py:
	cd py && tox -e integ -- tests/integ


# Run any development servers

run:
	cd js && npm start


# Build packages

build: build-js
.PHONY: build

build-js:
	cd js && npm run build

build-py:
	cd py && python3 setup.py build

build-r:
	cd r && R CMD build . && R CMD check *.tar.gz


# Generate documentation

docs: docs-js docs-py docs-r

docs-js:
	cd js && npm run docs

docs-py:
	cd py && echo "To do!"

docs-r:
	cd r && Rscript -e 'devtools::document()'

# Install packages

install: install-py install-r

install-py:
	cd py && pip install --user .

install-r: docs-r
	cd r && Rscript -e 'devtools::install()'

# Clean up local development environment

clean: clean-js clean-py

clean-js:
	cd js && npm run clean

clean-py:
	cd py && echo "To do!"
