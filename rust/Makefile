all: format lint cover audit build docs

setup:
	rustup component add clippy
	rustup toolchain install nightly
	cargo install \
		cargo-audit --features=fix
	cargo install \
		cargo-edit \
		cargo-strip \
		cargo-tarpaulin \
		cargo-udeps \
		cargo-watch
	cargo fetch

format:
	cargo fmt

lint:
	touch src/*.rs
	cargo clippy

test:
	cargo test

cover:
	cargo tarpaulin --ignore-tests --out Xml --out Html --output-dir coverage 

watch:
	cargo watch -x 'run -- serve'

audit:
	cargo +nightly udeps

build:
	cargo build --release && cargo strip

docs:
	cargo doc
.PHONY: docs
