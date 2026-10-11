.PHONY: fix lint test wasm

fix:
	cargo clippy --fix --allow-dirty --all-targets -p stencila-code-analysis -p stencila-graph
	cargo fmt -p stencila-code-analysis -p stencila-graph

lint:
	cargo clippy --all-targets -p stencila-code-analysis -p stencila-graph
	cargo fmt --check -p stencila-code-analysis -p stencila-graph

test:
	cargo test -p stencila-code-analysis -p stencila-graph

wasm:
	bash scripts/wasm-smoke.sh
