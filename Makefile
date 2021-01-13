all: format lint cover audit build docs

setup: setup-ts setup-rust
	
setup-ts:
	npm install

setup-rust:
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
	cargo clippy

test:
	cargo test

cover:
	cargo tarpaulin --ignore-tests --out Html --output-dir coverage

watch:
	cargo watch -x 'run -- serve --protocol=ws'

audit: audit-ts audit-rust

audit-ts:
	npm audit fix

audit-rust:
	cargo +nightly udeps
	cargo audit fix

build:
	cargo build --release && cargo strip

docs:
	cargo doc
.PHONY: docs

clean:
	cargo clean
