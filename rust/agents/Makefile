SPEC_URL := https://raw.githubusercontent.com/strongdm/attractor/refs/heads/main/coding-agent-loop-spec.md
SPEC_FILE := specs/coding-agent-loop-spec.md

.PHONY: update-spec spec-diff

spec-diff:
	mkdir -p specs
	tmp=$$(mktemp); \
	curl -fSL -o "$$tmp" $(SPEC_URL); \
	if [ -f "$(SPEC_FILE)" ]; then \
		git --no-pager diff --no-index --word-diff=color --unified=3 "$(SPEC_FILE)" "$$tmp" || true; \
	else \
		git --no-pager diff --no-index --word-diff=color --unified=3 /dev/null "$$tmp" || true; \
	fi; \
	rm -f "$$tmp"

update-spec:
	mkdir -p specs
	curl -fSL -o $(SPEC_FILE) $(SPEC_URL)
