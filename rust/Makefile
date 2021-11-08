all: fix lint cover audit build docs

setup:
	# Install cargo plugins
	cargo install \
		cargo-audit --features=fix
	cargo install \
		cargo-edit \
		cargo-strip \
		cargo-tarpaulin \
		cargo-udeps \
		cargo-watch
	# Nightly is required to run `cargo udeps`
	rustup toolchain install nightly

fix:
	cargo fmt --all
	cargo clippy --tests --fix --allow-dirty --all --exclude stencila-node

lint:
	cargo clippy --tests --all --exclude stencila-node

test:
	cargo test --all --exclude stencila-node

cover:
	cargo tarpaulin --ignore-tests --exclude-files tests/* \
	  --out Xml --out Html --output-dir coverage \
	  --timeout 600 --all --exclude stencila-node

audit:
	cargo +nightly udeps

build:
	cargo build --release

docs:
	cargo doc
.PHONY: docs
