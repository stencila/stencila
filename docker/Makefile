all: builder buildpacks stencila

builder:
	make -C builder/focal
.PHONY: builder

buildpacks:
	make -C buildpacks/focal
.PHONY: buildpacks

stencila:
	make -C stencila/focal
.PHONY: stencila
