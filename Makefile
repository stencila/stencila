all: setup lint test cover build docs

# Setup the local development environment

setup: setup-js setup-r

setup-js:
	cd js && npm install

setup-r:
	Rscript -e "install.packages('devtools')"
	Rscript -e "devtools::install_github(c('jimhester/lintr', 'klutometis/roxygen', 'r-lib/bench', 'r-lib/covr', 'r-lib/testthat'))"


# Add Git hooks

hooks:
	cp pre-commit.sh .git/hooks/pre-commit


# Lint code

lint: lint-js lint-r
	
lint-js:
	cd js && npm run lint

lint-r:
	cd r && Rscript -e 'lintr::lint_package()'

# Run tests

test: test-js test-r

test-js:
	cd js && npm test

test-r:
	cd r && Rscript -e 'devtools::test()'

# Run tests with coverage

cover: cover-js cover-r
	
cover-js:
	cd js && npm run cover

cover-r:
	cd r && Rscript -e 'covr::package_coverage()'

# Run benchmarks

bench: bench-js bench-r

bench-js:
	cd js && npm run bench

bench-r:
	cd r/tests/bench && Rscript all.R


# Run any development servers

run:
	cd js && npm start


# Build packages

build: build-js
.PHONY: build

build-js:
	cd js && npm run build

build-r:
	cd r && R CMD build . && R CMD check *.tar.gz


# Generate documentation

docs: docs-js docs-r

docs-js:
	cd js && npm run docs

docs-r:
	cd r && Rscript -e 'devtools::document()'

# Install packages

install: install-r

install-r: docs-r
	cd r && Rscript -e 'devtools::install()'

# Clean up local development environment

clean: clean-js

clean-js:
	cd js && npm run clean
