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

# Build requires the `web` module's JavaScript to be built
# so that it can be embedded in the binary.
build:
	make -C ../web build
	cargo build --release
	cargo strip

install:
	cargo install --path .
