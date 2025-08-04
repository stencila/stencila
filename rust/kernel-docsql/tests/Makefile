# Makefile for kernel-docsql tests

.PHONY: clean test

# Clean up generated JSON files from API responses
clean:
	rm -f *.json

# Run all golden tests
test:
	cargo test -p kernel-docsql golden

# Run tests for specific API types
test-openalex:
	GLOB="*.openalex" cargo test -p kernel-docsql golden

test-github:
	GLOB="*.github" cargo test -p kernel-docsql golden

test-zenodo:
	GLOB="*.zenodo" cargo test -p kernel-docsql golden

# Run tests without HTTP requests (for CI)
test-no-http:
	NO_HTTP=1 cargo test -p kernel-docsql golden

# Update golden test expectations
update:
	UPDATE_GOLDEN=1 cargo test -p kernel-docsql golden