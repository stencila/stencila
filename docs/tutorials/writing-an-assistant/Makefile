.DEFAULT_GOAL := test
SHELL := /bin/bash

.PHONY: test
test:
	STENCILA_PROMPTS_DIR=./prompts cargo run --bin stencila test examples

.PHONY: list
list:
	STENCILA_PROMPTS_DIR=./prompts cargo run --bin stencila prompts

.PHONY: clean
clean:
	rm -fv examples/*.*.{json,md}
