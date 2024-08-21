.DEFAULT_GOAL := test
SHELL := /bin/bash

.PHONY: test
test:
	STENCILA_PROMPTS_DIR=./prompts cargo run -p cli test examples

.PHONY: list
list:
	STENCILA_PROMPTS_DIR=./prompts cargo run -p cli prompts

.PHONY: clean
clean:
	rm -fv examples/*.*.{json,md}
