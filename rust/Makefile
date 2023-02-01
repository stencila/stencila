all: fix lint cover outdated audit build docs

ifdef CI
  PROFILE:=ci
else
  PROFILE:=dev
endif 

# Install Rust tools used during development
# Nightly toolchain is required to run `cargo udeps`
setup:
	cargo install cargo-audit --features=fix
	cargo install cargo-watch cargo-insta
	rustup toolchain install nightly
	cargo install cargo-udeps --locked
	cargo install cargo-outdated --locked

# Run formatting and linting fixes
format:
	cargo fmt --all
	cargo clippy --workspace --all-features --tests --fix --allow-dirty

# Run clippy linting
lint:
	cargo clippy --workspace --all-features --tests

# Run tests
test:
	cargo test --workspace --all-features --profile $(PROFILE) --no-fail-fast -- --nocapture

# List outdated dependencies
outdated:
	cargo outdated --workspace --all-features

# Audit for unused dependencies
audit:
	cargo +nightly udeps --workspace --all-features

# Build binary
build:
	cargo build --bin stencila --release

# Build `musl` binary
build-musl:
	cargo build --bin stencila --release --target x86_64-unknown-linux-musl

# Use cargo to install the binary
install:
	cargo install --path stencila

# Generate Rust schema types
schema:
	cargo run -p schema-gen -- --rust
.PHONY: schema

# Generate docs
# Runs internal document generation crates
docs:
	cargo run -p schema-gen -- --docs
