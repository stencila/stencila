all: regen format lint test

regen:
	npm run build:rust

format:
	cargo fmt

lint:
	cargo clippy

test:
	cargo test
