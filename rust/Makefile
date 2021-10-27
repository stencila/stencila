all: format lint cover audit build docs

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
	cargo fmt
	cargo clippy --tests --fix --allow-dirty

lint:
	cargo clippy --tests

test:
	cargo test

cover:
	cargo tarpaulin --ignore-tests --exclude-files tests/* \
	  --out Xml --out Html --output-dir coverage \
	  --timeout 600

audit:
	cargo +nightly udeps

build:
	cargo build --release

docs:
	cargo doc
.PHONY: docs
