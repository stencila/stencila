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
		cargo-watch \
		cargo-insta \
		libcnb-cargo
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

# Build and strip binary
# Builds / installs embedded static assets first
build:
	cd ../schema && npm install && npm run build
	cd ../node && npm install && npm run build:ts
	cd ../web && npm install && npm run build
	cd ../themes && npm install && npm run build
	cd stencila && npm install
	cargo build --bin stencila --release
	cd .. && cargo strip

# Build and strip `musl` binaries
build-musl:
	cargo build --bin stencila --release --target x86_64-unknown-linux-musl
	cd .. && cargo strip --target x86_64-unknown-linux-musl

install:
	cargo install --path stencila


buildpack-%: FORCE
	cd $@ && cargo build --release
FORCE:

docs:
	cargo doc
.PHONY: docs
