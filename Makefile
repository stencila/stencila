# What? A Makefile for running high level development tasks. For finer
# grained tasks see `package.json` and use `npm run <task>`.
#
# Why? Because Makefiles provide a common entry point for developers
# that are independent of the language and tooling used. In just
# about all our repos you can run `make setup` to get a dev setup
# and `cat Makefile` to quickly understand what to do next. Regardless
# or whether it's a Typescript, Python or R project.

all: lint test build docs

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
