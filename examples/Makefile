SHELL := /bin/bash

# Test a particular assistant
assistants/%:
	cargo run -p cli test $@

# Test all assistants
assistants: $(wildcard assistants/*/)

# Clean up generated but not commited files
clean:
	rm -fv assistants/**/*.*.{json,md}
