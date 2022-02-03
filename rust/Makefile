all: fix lint cover audit build docs

setup:
	# Install cargo plugins
	cargo install \
		cargo-audit --features=fix
	cargo install \
		cargo-edit \
		cargo-insta \
		cargo-strip \
		cargo-tarpaulin \
		cargo-udeps \
		cargo-watch
	# Nightly is required to run `cargo udeps`
	rustup toolchain install nightly

fix:
	cargo fmt --all
	cargo clippy --workspace --exclude stencila-node --all-features --tests --fix --allow-dirty

lint:
	cargo clippy --workspace --exclude stencila-node --all-features --tests

test:
	cargo test --workspace --exclude stencila-node --all-features -- --nocapture

cover:
	cargo tarpaulin --ignore-tests --exclude-files tests/* \
	  --out Xml --out Html --output-dir coverage \
	  --timeout 600 \
	  --workspace --exclude stencila-node --all-features

audit:
	cargo +nightly udeps --workspace --exclude stencila-node --all-features

build:
	# Build / install embedded static assets
	cd ../web && npm install && npm run build
	cd ../themes && npm install && npm run build
	cd stencila && npm install
	# Build and strip binary
	cargo build --bin stencila --release
	cd .. && cargo strip

install:
	cargo install --path stencila

docs:
	cargo doc
.PHONY: docs
