all: format lint cover outdated audit build docs

ifdef CI
  PROFILE:=ci
else
  PROFILE:=dev
endif 

# Install Rust tools used during development
# Nightly toolchain is required to run `cargo udeps`
setup:
	cargo install cargo-binstall
	cargo binstall --no-confirm cargo-audit cargo-dist cargo-insta cargo-outdated cargo-release cargo-watch
	rustup toolchain install nightly
	cargo install cargo-udeps --locked

# Run formatting and linting fixes
fix:
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

# Audit dependencies
audit:
	cargo +nightly udeps --workspace --all-features
	cargo audit -f ../Cargo.lock

# Build CLI binary
cli:
	cargo build --bin stencila --release

# Build `musl` CLI binary
cli-musl:
	cargo build --bin stencila --release --target x86_64-unknown-linux-musl

# Use cargo to install the binary
install:
	cargo install --path stencila

# Build generated files
generated:
	cargo run -p schema-gen

# Update derived examples
examples:
	UPDATE_EXAMPLES=true cargo test -p tests examples_encode_decode

# Clean the ../target dir
clean:
	cargo clean --target-dir ../target
