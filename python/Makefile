all: fix audit test build

install:
	make -C stencila install
	make -C stencila_types install
	make -C stencila_plugin install

lint:
	make -C stencila lint
	make -C stencila_types lint
	make -C stencila_plugin lint

audit:
	make -C stencila audit
	make -C stencila_types audit
	make -C stencila_plugin audit

test:
	make -C stencila test
	make -C stencila_types test
	make -C stencila_plugin test

build:
	make -C stencila build
	make -C stencila_types build
	make -C stencila_plugin build

fix:
	make -C stencila fix
	make -C stencila_types fix
	make -C stencila_plugin fix
