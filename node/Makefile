all: format lint test build

setup:
	npm install

format:
	cargo fmt --manifest-path native/Cargo.toml

lint:
	touch native/src/*.rs
	cargo clippy

test:
	npm run test

audit:
	cargo +nightly udeps

build:
	npm run build

clean:
	npm run clean
