# A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`, or the
# `Makefiles` for each language folder e.g. `py/Makefile`.

all: format lint test build docs

setup:
	npm install
	make -C py setup
	make -C r setup

format:
	npm run format

lint:
	npm run lint
	make -C py lint
	make -C r lint

test:
	npm test
	make -C py test
	make -C r test

build:
	npm run build
	make -C py build
	make -C r build

docs:
	npm run docs
	make -C py docs
	make -C r docs
.PHONY: docs

clean:
	npm run clean
	make -C py clean
	make -C r clean

PYBINDINGS := 'py/stencila/schema/types.py'
RBINDINGS := 'r/R/types.R'
RNAMESPACE := 'r/NAMESPACE'

# Build schema bindings for Python and R. If the bindings have changed, this script
# will commit the updated bindings. This script is run automatically as a git pre-push hook.
check-bindings:
	@echo "🔬 Checking if schema bindings have changed"
	@echo "🏗 Building schema language bindings"
	npm run build:jsonschema
	npm run build:py & npm run build:r
	@echo "🔗 Finished building language bindings"
	@for i in $$(git ls-files -m); do \
		if [ "$$i" = $(PYBINDINGS) ] || [ "$$i" = $(RBINDINGS) ] ; then \
			echo "☝️ Bindings have changed, please verify and commit them."; \
			echo "If there are no other changes, you can run \"make commit-bindings\"\n\n"; \
			exit 1; \
		fi \
	done

# Commits just the updated schema bindings
commit-bindings:
	git commit --only $(PYBINDINGS) $(RBINDINGS) $(RNAMESPACE) -m "chore(Language bindings): Update type bindings"


# Build Docker image for development
build-image:
	docker build \
	  --build-arg USER_ID=$$(id -u) \
    --build-arg GROUP_ID=$$(id -g) \
		-t stencila/schema .

# Run an interactive shell in Docker container
run-image:
	docker run --rm -it -v $$PWD:/code -w /code stencila/schema bash
