all: format lint build

format:
	cargo fmt

lint:
	touch src/*.rs
	cargo clippy

build:
	cargo build --release
