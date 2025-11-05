# Linting checks
lint:
	shellcheck defaults/*.sh

# Build the image
build:
	docker build --platform linux/amd64 --tag stencila/workspace .

# Build the image from scratch
build-no-cache:
	docker build --no-cache --platform linux/amd64 --tag stencila/workspace .

# Run the image as a cloud development environment
# Navigate to http://localhost:8080/?folder=/home/workspace/stencila/test to open test repo
cde: build
	docker run -it --rm --publish 8080:8080 \
		-e GITHUB_REPO=stencila/test \
		stencila/workspace

# Run the image as a continuous integration job
# (don't run any script, just test setup)
# Usage: make ci [GITHUB_REPO=owner/repo] [REPO_REF=branch-name]
ci: build
	docker run --rm \
		-e GITHUB_REPO=$(if $(GITHUB_REPO),$(GITHUB_REPO),stencila/test) \
		-e STENCILA_SCRIPT=none \
		$(if $(REPO_REF),-e REPO_REF=$(REPO_REF)) \
		stencila/workspace

# Debug the image by entering a bash shell within it
debug: build
	docker run -it --rm --entrypoint /bin/bash stencila/workspace

# Publish the image
publish: build
	@TIMESTAMP=$$(date -u +"%Y-%m-%dT%H-%M-%SZ") ; \
	docker tag stencila/workspace:latest stencila/workspace:$$TIMESTAMP ; \
	docker push stencila/workspace:latest ; \
	docker push stencila/workspace:$$TIMESTAMP
