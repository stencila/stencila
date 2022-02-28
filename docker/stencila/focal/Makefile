# Build the image
build:
	docker build --tag stencila/stencila:focal --tag stencila/stencila:latest .

# Test that the image will at least run (i.e. has necessary system dependencies)
test:
	docker run --rm -it stencila/stencila:focal --version
