all: format lint cover audit build docs

# Some Cargo commands which only make sense at this top level of the
# Cargo workspace (e.g. `cargo clean`, `cargo audit fix`) are added below
# (in addition to the cargo commands in the Makefile for each language package)

format:
	make -C rust format
	make -C cli format
	make -C node format

lint:
	make -C rust lint
	make -C cli lint
	make -C node lint

test:
	make -C rust test
	make -C node test

cover:
	make -C rust cover
	make -C node cover

# Watch for changes, recompile and serve
# Useful for things like quickly previewing changes in HTML encoding
# After visiting login page, open browser at http://127.0.0.1:9000/fixtures/articles/elife-small.json for example.
# Uses a fixed key to avoid having to relogin on each reload.
# You can use --insecure if your prefer.
watch-serve:
	cargo watch -x 'run -- serve --debug --key=$(shell openssl rand -hex 12)'

audit:
	make -C rust audit
	make -C node audit
	cargo audit fix

build:
	make -C rust build
	make -C cli build
	make -C node build

docs:
	make -C rust docs
	make -C node docs
.PHONY: docs

clean:
	cargo clean
