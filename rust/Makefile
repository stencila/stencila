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
	cargo tarpaulin --ignore-tests --exclude-files tests/* --out Xml --out Html --output-dir coverage 

audit:
	cargo +nightly udeps

build:
	cargo build --release

install:
	cargo install --path .

docs:
	cargo doc
.PHONY: docs
