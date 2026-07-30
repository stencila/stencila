all: generated lint test

# Regenerate skill references from site/docs
generated:
	./scripts/gen-references.sh

# Validate plugin and marketplace manifests and check for CLI flag drift.
# Tool availability is checked so the aggregate `make lint` at the repo root
# does not fail on machines without the claude or stencila CLIs; the
# dedicated `claude` CI job enforces these checks hard by invoking the
# commands and scripts directly.
lint:
	@if command -v claude > /dev/null 2>&1; then \
		claude plugin validate . --strict && claude plugin validate ..; \
	else \
		echo "lint: claude CLI not found; skipping manifest validation"; \
	fi
	@if [ -x ../target/debug/stencila ] || command -v stencila > /dev/null 2>&1; then \
		./scripts/check-flags.sh; \
	else \
		echo "lint: stencila CLI not found; skipping flag drift check"; \
	fi

# Parse every smd example embedded in the skills
test:
	@if [ -x ../target/debug/stencila ] || command -v stencila > /dev/null 2>&1; then \
		./scripts/test-examples.sh; \
	else \
		echo "test: stencila CLI not found; skipping smd example tests"; \
	fi
