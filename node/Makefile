all: fix audit test build

install:
	make -C stencila-node install
	make -C stencila-plugin install

fix:
	make -C stencila-node fix
	make -C stencila-plugin fix

lint:
	make -C stencila-node lint
	make -C stencila-plugin lint

outdated:
	make -C stencila-node outdated
	make -C stencila-plugin outdated

audit:
	make -C stencila-node audit
	make -C stencila-plugin audit

pubcheck:
	make -C stencila-node pubcheck
	make -C stencila-plugin pubcheck

test:
	make -C stencila-node test

cover:
	make -C stencila-node cover

build:
	make -C stencila-node build

clean:
	make -C stencila-node clean
