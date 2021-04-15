all: format lint test build

node_modules:
	npm install

setup: node_modules

format:
	cargo fmt --manifest-path native/Cargo.toml

lint:
	touch native/src/*.rs
	cargo clippy

test: setup
	npm run test

audit:
	cargo +nightly udeps

build: setup
	npm run build

docs: setup
	npm run docs

clean:
	npm run clean
