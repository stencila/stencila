# What? A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`, or the
# `Makefiles` for each language folder e.g. `py/Makefile`.

all: lint test build docs

setup:
	npm install
	make -C py setup
	make -C r setup

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

# Build schema bindings for Python and R. If the bindings have changed, this script
# will commit the updated bindings. This script is run automatically as a git pre-push hook.
checkBindings:
	@echo "🔬 Checking if schema bindings have changed"
	@echo "🏗 Building schema language bindings"
	npm run build:jsonschema
	npm run build:py & npm run build:r
	@echo "🔗 Finished building language bindings"
	@for i in $$(git ls-files -m); do \
		if [ "$$i" = $(PYBINDINGS) ] || [ "$$i" = $(RBINDINGS) ] ; then \
			echo "☝️ Bindings have changed, please verify and commit them."; \
			echo "If there are no other changes, you can run \"make commitBindings\"\n\n"; \
			exit 1; \
		fi \
	done

## Commits just the updated schema bindings
commitBindings:
	git commit --only $(PYBINDINGS) $(RBINDINGS) -m "chore(Type Bindings): Generate updated bindings"
