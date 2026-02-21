SPEC_URL := https://raw.githubusercontent.com/agentskills/agentskills/refs/heads/main/docs/specification.mdx
SPEC_FILE := specs/agent-skills.md

CRATE := stencila-skills

.PHONY: lint format test check spec-update spec-diff

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

lint:
	cargo clippy --fix --allow-dirty --all-targets -p $(CRATE)

format:
	cargo fmt -p $(CRATE)

test:
	cargo test --all-features -p $(CRATE)

check: lint format test
