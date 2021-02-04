all: format lint build

setup:
	npm install

format:
	cargo fmt --manifest-path native/Cargo.toml

lint:
	touch native/src/*.rs && cargo clippy

build:
	npx neon build --release

clean:
	npx neon clean
