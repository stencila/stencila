# Build the binary, copy it into this folder and then
# build the `Dockerfile` (which copies it into the image)
# Designed to only be done on Linux (an alternative would be to have
# a two stage Dockerfile in which the binary is build in the first
# stage and copied to the second stage).
build:
	make -C ../../rust build
	cp ../../target/release/stencila .
	docker build --tag stencila/stencila .

# Test that the image will run (i.e. has necessary system dependencies)
test:
	docker run --rm -it stencila/stencila help
