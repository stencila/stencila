all: format lint audit build

format:
	cargo fmt

lint:
	touch *.rs
	cargo clippy

audit:
	cargo +nightly udeps

build:
	cargo build --release
	cargo strip
