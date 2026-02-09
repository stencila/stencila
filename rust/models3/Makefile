SPEC_URL := https://raw.githubusercontent.com/strongdm/attractor/refs/heads/main/unified-llm-spec.md
SPEC_FILE := specs/unified-llm-spec.md

.PHONY: update-spec

update-spec:
	mkdir -p specs
	curl -fSL -o $(SPEC_FILE) $(SPEC_URL)
