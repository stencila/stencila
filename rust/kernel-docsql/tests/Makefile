# Makefile for kernel-docsql tests

.PHONY: clean test test-with-queries

# Clean up generated JSON files from API responses
clean:
	rm -f *.json

# Run all golden tests (default: no API queries executed)
test:
	cargo test -p kernel-docsql golden

# Run golden tests with actual API query execution and JSON generation
test-with-queries:
	EXECUTE_QUERIES=1 cargo test -p kernel-docsql golden

# Run tests for specific API types (no queries)
test-openalex:
	GLOB="*.openalex" cargo test -p kernel-docsql golden

test-github:
	GLOB="*.github" cargo test -p kernel-docsql golden

test-zenodo:
	GLOB="*.zenodo" cargo test -p kernel-docsql golden

# Run tests for specific API types with queries
test-openalex-with-queries:
	EXECUTE_QUERIES=1 GLOB="*.openalex" cargo test -p kernel-docsql golden

test-github-with-queries:
	EXECUTE_QUERIES=1 GLOB="*.github" cargo test -p kernel-docsql golden

test-zenodo-with-queries:
	EXECUTE_QUERIES=1 GLOB="*.zenodo" cargo test -p kernel-docsql golden

# Update golden test expectations
update:
	UPDATE_GOLDEN=1 cargo test -p kernel-docsql golden