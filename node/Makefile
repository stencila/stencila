all: format lint test build docs

node_modules:
	npm install

setup: node_modules

format:
	cargo fmt

lint:
	touch src/*.rs
	cargo clippy

test: setup
	npm run test

cover: setup
	npm run cover

audit:
	cargo +nightly udeps

try:
	npm run build:rust:debug
	npm run build:ts
	node

build: setup
	npm run build

docs: setup
	npm run docs

clean:
	npm run clean
