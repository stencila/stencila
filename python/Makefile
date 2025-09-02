all: fix audit test build

install:
	make -C stencila install
	make -C stencila_types install

lint:
	make -C stencila lint
	make -C stencila_types lint

audit:
	make -C stencila audit
	make -C stencila_types audit

test:
	make -C stencila test
	make -C stencila_types test

build:
	make -C stencila build
	make -C stencila_types build

fix:
	make -C stencila fix
	make -C stencila_types fix
