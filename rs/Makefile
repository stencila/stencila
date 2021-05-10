all: regen format lint test docs

regen:
	npm run build:rust

format:
	cargo fmt

lint:
	cargo clippy

test:
	cargo test

docs:
	cargo doc --no-deps

clean:
	cargo clean
