SHELL := /bin/bash

all: codecs kernels

# Execute a particular prompt example and save as JSON
prompts/%.json: prompts/%.md
	cargo run -p cli execute $< $@ --skip-code

# Execute all prompt examples
prompts: $(patsubst %.md,%.json,$(filter-out %README.md,$(wildcard prompts/**/*.md)))
.PHONY: prompts

# Run the codecs `examples` test with `UPDATE_EXAMPLES` env var to
# overwrite existing generated files
codecs:
	UPDATE_EXAMPLES=true cargo test -p codecs --test examples

# Execute a particular kernel example and save as JSON
kernels/%.json: kernels/%.md
	cargo run -p cli --quiet execute $< $@

# Execute all kernel examples
kernels: $(patsubst %.md,%.json,$(filter-out %README.md,$(wildcard kernels/**/*.md)))
.PHONY: kernels

# Clean up generated but not commited files
clean:
	rm -fv prompts/**/*.json kernels/**/*.json
