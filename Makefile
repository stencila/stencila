all: format lint cover audit build docs

# Some Cargo commands which only make sense at this top level (e.g. `clean`)
# of the Cargo workspace are added below, in addition to those in the
# `rust` package's `Makefile`.

format:
	make -C rust format

lint:
	make -C rust lint

test:
	make -C rust test

cover:
	make -C rust cover

watch:
	make -C rust watch

audit:
	make -C rust audit
	cargo audit fix

build:
	make -C rust build
	cargo strip

docs:
	make -C rust docs

clean:
	cargo clean
