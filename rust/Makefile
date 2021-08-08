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

format:
	cargo fmt

lint:
	touch src/*.rs
	cargo clippy

test:
	cargo test

cover:
	cargo tarpaulin --ignore-tests --exclude-files tests/* --out Xml --out Html --output-dir coverage 

audit:
	cargo +nightly udeps

build:
	cargo build --release

docs:
	cargo doc
.PHONY: docs
