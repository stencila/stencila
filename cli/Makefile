all: format lint audit build

format:
	cargo fmt

lint:
	touch *.rs
	cargo clippy

test:
	cargo test

cover:
	cargo tarpaulin --ignore-tests --out Xml --out Html --output-dir coverages

audit:
	cargo +nightly udeps

build:
	cargo build --release
	cargo strip
