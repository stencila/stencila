SHELL := /bin/bash

all: codecs kernels

# Execute a particular assistant example and save as JSON
assistants/%.json: assistants/%.md
	cargo run -p cli execute $< $@ --skip-code

# Execute all assistant examples
assistants: $(patsubst %.md,%.json,$(filter-out %README.md,$(wildcard assistants/**/*.md)))
.PHONY: assistants

# Run the codecs `examples` test with `UPDATE_EXAMPLES` env var to
# overwrite existing generated files
codecs:
	UPDATE_EXAMPLES=true cargo test -p codecs --test examples

# Execute a particular kernel example and save as JSON
kernels/%.json: kernels/%.md
	cargo run -p cli execute $< $@

# Execute all kernel examples
kernels: $(patsubst %.md,%.json,$(filter-out %README.md,$(wildcard kernels/**/*.md)))
.PHONY: kernels

# Clean up generated but not commited files
clean:
	rm -fv assistants/**/*.json kernels/**/*.json
