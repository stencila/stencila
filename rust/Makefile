all: fix lint cover audit build docs

setup:
	# Install cargo plugins
	cargo install \
		cargo-audit --features=fix
	cargo install \
		cargo-edit \
		cargo-insta \
		cargo-strip \
		cargo-tarpaulin \
		cargo-udeps \
		cargo-watch \
		cargo-insta \
		libcnb-cargo
	# Nightly is required to run `cargo udeps`
	rustup toolchain install nightly

fix:
	cargo fmt --all
	cargo clippy --workspace --exclude stencila-node --all-features --tests --fix --allow-dirty

lint:
	cargo clippy --workspace --exclude stencila-node --all-features --tests

test:
	cargo test --workspace --exclude stencila-node --all-features -- --nocapture

cover:
	cargo tarpaulin --ignore-tests --exclude-files tests/* \
	  --out Xml --out Html --output-dir coverage \
	  --timeout 600 \
	  --workspace --exclude stencila-node --all-features

audit:
	cargo +nightly udeps --workspace --exclude stencila-node --all-features

# Build and strip binary
# Builds / installs embedded static assets first
build:
	cd ../schema && npm install --force && npm run build
	cd ../web && npm install && npm run build
	cd ../themes && npm install && npm run build
	cd server-next && npm install
	cargo build --bin stencila --release
	cd .. && cargo strip

# Build and strip `musl` binaries
build-musl:
	cargo build --bin stencila --release --target x86_64-unknown-linux-musl
	cd .. && cargo strip --target x86_64-unknown-linux-musl

install:
	cargo install --path stencila

# Update the `versions.rs` file for a binary
binary-%-versions:
	cargo run --bin binaries --all-features -- versions $* --write binary-$*/src/versions.rs

# Update the `version.rs` files for all binaries that have one
binaries-versions: binary-asdf-versions binary-node-versions binary-pack-versions \
                   binary-pandoc-versions binary-podman-versions binary-poetry-versions \
				   binary-python-versions binary-r-versions binary-stencila-versions

# Package a buildpack
buildpack-%-debug:
	cd buildpack-$* && cargo libcnb package

# Package a buildpack release
buildpack-%: FORCE
	cd buildpack-$* && cargo libcnb package --release
FORCE:

docs:
	cargo doc
.PHONY: docs
