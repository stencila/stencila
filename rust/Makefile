all: fix test audit build

ifdef CI
  PROFILE:=ci
else
  PROFILE:=dev
endif 

# Setup local development environment
# Nightly toolchain is required to run `cargo udeps`
setup:
	cargo install cargo-binstall
	cargo binstall --no-confirm cargo-audit cargo-dist cargo-insta cargo-outdated cargo-release cargo-watch
	rustup toolchain install nightly
	cargo install cargo-udeps --locked

# Make formatting and linting fixes
fix:
	cargo fmt --all
	cargo clippy --workspace --all-features --tests --fix --allow-dirty

# Run linting checks
lint:
	cargo clippy --workspace --all-features --tests

# Run tests
test:
	cargo test --workspace --all-features --profile $(PROFILE) --no-fail-fast -- --nocapture

# List outdated dependencies
outdated:
	cargo outdated -d 1

# Audit dependencies
audit:
	cargo +nightly udeps --workspace --all-targets --all-features
	cargo audit -f ../Cargo.lock

# Build CLI binary
build:
	cargo build --bin stencila --release

# Use cargo to install the binary
install:
	cargo install --path stencila

# Build generated files
generated:
	cargo run -p schema-gen
	cargo run -p stencila-gen > ../docs/reference/cli.md

# Update derived examples
examples:
	UPDATE_EXAMPLES=true cargo test -p tests examples_encode_decode

# Clean the ../target dir
clean:
	cargo clean --target-dir ../target
