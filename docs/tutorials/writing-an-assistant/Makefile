.DEFAULT_GOAL := test
SHELL := /bin/bash

.PHONY: test
test:
	STENCILA_ASSISTANTS_DIR=./assistants cargo run -p cli test examples

.PHONY: list
list:
	STENCILA_ASSISTANTS_DIR=./assistants cargo run -p cli assistants

.PHONY: clean
clean:
	rm -fv examples/*.*.{json,md}
