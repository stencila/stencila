all: format lint test build docs

node_modules: package.json
	npm install

setup: node_modules

format:
	cargo fmt

lint:
	touch src/*.rs
	cargo clippy

build-debug: setup
	npm run build:rust:debug

build-ts: setup
	npm run build:ts

build-types: build-debug
	npm run build:types

test: setup build-debug
	npm run test

cover: setup build-debug
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
