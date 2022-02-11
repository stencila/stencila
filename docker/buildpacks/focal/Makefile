# Build the `build` and `run` images
build:
	docker build --target build --tag stencila/build:focal --tag stencila/build:latest .
	docker build --target run --tag stencila/run:focal --tag stencila/run:latest .
	pack builder create stencila/buildpacks:focal --config ./builder.toml
