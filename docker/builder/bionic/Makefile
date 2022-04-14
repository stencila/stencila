build: create

# Build an individual buildpack by delegating to the Rust Makefile
buildpack-%:
	make -C ../../../rust $@
	
# Build the buildpacks that are included in the builder
buildpacks: buildpack-apt buildpack-node buildpack-python buildpack-r buildpack-stencila

# Create the builder using pack
create: buildpacks
	pack builder create stencila/builder:bionic --config ./builder.toml
