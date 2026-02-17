SPEC_URL := https://raw.githubusercontent.com/strongdm/attractor/refs/heads/main/attractor-spec.md
SPEC_FILE := specs/attractor-spec.md

CRATE := stencila-attractor

.PHONY: fmt lint test check spec-update spec-diff

fmt:
	cargo fmt -p $(CRATE)

lint:
	cargo clippy --fix --allow-dirty --all-targets -p $(CRATE)

test:
	cargo test -p $(CRATE)

check: fmt lint test

spec-diff:
	@mkdir -p specs
	@tmp=$$(mktemp); \
	curl -fsSL -o "$$tmp" $(SPEC_URL); \
	if [ -f "$(SPEC_FILE)" ]; then \
		git --no-pager diff --no-index --word-diff=color --unified=3 "$(SPEC_FILE)" "$$tmp" || true; \
	else \
		git --no-pager diff --no-index --word-diff=color --unified=3 /dev/null "$$tmp" || true; \
	fi; \
	rm -f "$$tmp"

spec-update:
	mkdir -p specs
	curl -fSL -o $(SPEC_FILE) $(SPEC_URL)
