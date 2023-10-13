all: fix test audit build

# Rust profile
ifdef CI
  PROFILE:=ci
else
  PROFILE:=dev
endif

# Rust target defaults
# Can be overriden e.g.
#   make build -e TARGET=x86_64-apple-darwin
ifeq ($(OS),Windows_NT)
    TARGET:=x86_64-pc-windows-gnu
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        TARGET:=x86_64-unknown-linux-gnu
    endif
    ifeq ($(UNAME_S),Darwin)
        TARGET:=x86_64-apple-darwin
    endif
endif

# Setup local development environment
# Nightly toolchain is required to run `cargo udeps`
setup:
	cargo install cargo-binstall
	cargo binstall --no-confirm cargo-audit cargo-insta cargo-outdated cargo-watch
	rustup toolchain install nightly
	cargo install cargo-udeps --locked

# Make formatting and linting fixes
fix:
	cargo clippy --workspace --tests --fix --allow-dirty
	cargo fmt --all

# Run linting checks
lint:
	cargo clippy --workspace --tests

# Run tests
test:
	cargo test --workspace --profile $(PROFILE) --no-fail-fast -- --nocapture
	make proptest
	make a11ytest

# Run all property tests at all levels
proptest: proptest-min proptest-low proptest-high proptest-max

proptest-min:
	cargo test --profile $(PROFILE) -p codecs --features proptest-min

proptest-low:
	cargo test --profile $(PROFILE) -p codecs --features proptest-low

proptest-high:
	cargo test --profile $(PROFILE) -p codecs --features proptest-high

proptest-max:
	cargo test --profile $(PROFILE) -p codecs --features proptest-max

# Run accessibility tests
a11ytest:
	cd codecs/src/tests && npm install && npm run test:a11y

# List outdated dependencies
outdated:
	cargo outdated -d 1

# Audit dependencies
audit:
	cargo audit -f ../Cargo.lock

# List unused dependencies
# Not included in `audit` recipe (as in SDK Makefiles) because
# requires nightly Rust compiler and so complicates running audit on CI
unused:
	cargo +nightly udeps --workspace --all-targets

# Build CLI binary
build:
	cargo build --bin stencila --release --target $(TARGET)

# Create archive of CLI binary and other assets
archive:
	cd .. && bash scripts/archive-binary.sh cli $(TARGET) stencila

# Use cargo to install the CLI binary
install:
	cargo install --path cli

# Build generated files
generated:
	cargo run -p schema-gen
	cargo run -p cli-gen > ../docs/reference/cli.md

# Update derived examples
examples:
	UPDATE_EXAMPLES=true cargo test -p tests examples_encode_decode

# Clean the ../target dir
clean:
	cargo clean --target-dir ../target
