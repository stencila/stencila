all: fix test audit build

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
	cargo binstall --no-confirm cargo-audit cargo-insta cargo-llvm-cov cargo-outdated cargo-watch
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
	cargo test --workspace --no-fail-fast -- --nocapture
	make proptest

# Run tests with coverage
# Coverage is reported to ../lcov.info so it can be used by the VSCode Coverage Gutters extension
cover:
	cargo llvm-cov clean --workspace 
	cargo llvm-cov test --workspace  --no-report
	# SKIP: NO TESTS cargo llvm-cov test -p codecs --features proptest-min --no-report proptests
	# SKIP: NO TESTS cargo llvm-cov test -p codecs --features proptest-low --no-report proptests
	cargo llvm-cov test -p codecs --features proptest-high --no-report proptests
	cargo llvm-cov test -p codecs --features proptest-max --no-report proptests
	cd .. && cargo llvm-cov report --lcov --output-path lcov.info --ignore-filename-regex '(test)|(rust/schema-gen)|(rust/schema/src/types)|(python/src)|(node/src)'

# Run all property tests at all levels
proptest: proptest-min proptest-low proptest-high proptest-max

proptest-min:
	cargo test -p codecs --features proptest-min proptests

proptest-low:
	cargo test -p codecs --features proptest-low proptests

proptest-high:
	cargo test -p codecs --features proptest-high proptests

proptest-max:
	cargo test -p codecs --features proptest-max proptests

# Run accessibility tests
a11y:
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

# Update files generated for examples
examples:
	UPDATE_EXAMPLES=true cargo test -p codecs examples

# Clean the ../target dir
clean:
	cargo clean --target-dir ../target
