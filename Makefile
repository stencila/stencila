all: format lint cover audit build docs

# Some Cargo commands which only make sense at this top level of the
# Cargo workspace (e.g. `cargo clean`, `cargo audit fix`) are added below
# (in addition to the cargo commands in the Makefile for each language package)

setup:
	make -C rust setup

format:
	make -C rust format

lint:
	make -C rust lint

test:
	make -C rust test

cover:
	make -C rust cover

audit:
	cargo audit fix

build:
	make -C rust build

docs:
	make -C rust docs
.PHONY: docs

clean:
	cargo clean
