# Linting checks
lint:
	shellcheck defaults/setup.sh

# Build the image
build:
	docker build --platform linux/amd64 --tag stencila/workspace .

# Build the image from scratch
build-no-cache:
	docker build --no-cache --platform linux/amd64 --tag stencila/workspace .

# Run the image as a server
run: build
	docker run -it --rm --privileged --platform linux/amd64 --publish 8080:8080 stencila/workspace

# Debug the image by entering a bash shell within it
debug: build
	docker run -it --rm --privileged --platform linux/amd64 --entrypoint /bin/bash stencila/workspace

# Publish the image
publish: build
	@TIMESTAMP=$$(date -u +"%Y-%m-%dT%H-%M-%SZ") ; \
	docker tag stencila/workspace:latest stencila/workspace:$$TIMESTAMP ; \
	docker push stencila/workspace:latest ; \
	docker push stencila/workspace:$$TIMESTAMP
