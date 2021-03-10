# Build the image
build:
	docker build --tag stencila/stencila .

# Test that the image will run (i.e. has necessary system dependencies)
test:
	docker run --rm -it stencila/stencila help
