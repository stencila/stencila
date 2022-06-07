# Stencila Cloud Native Buildpacks

## Introduction

Stencila converts your project source code into a container image using [Cloud Native Buildpacks](https://buildpacks.io/docs/concepts/). This Rust crate pulls together Stencila buildpacks and provides a CLI interface for inspecting them and previewing build plans.

## Development

### Setup

Stencila buildpacks are developed using [`libcnb.rs`](https://github.com/Malax/libcnb.rs). See the [instructions for setting up a development environment](https://github.com/Malax/libcnb.rs#development-environment-setup) to build `libcnb.rs`-based buildpacks locally e.g.

```sh
cargo install libcnb-cargo
rustup target add x86_64-unknown-linux-musl
```

### Developing individual buildpacks

The best place to start is to look at some of the existing buildpacks to get an idea of the required files and APIs. Once your buildpack is setup and common workflow is to `cd` into its folder and package it in debug mode:

```sh
cd rust/buildpack-python
cargo libcnb package
```

Then run the buildpack by running `pack` on a fixture,

```sh
pack build --buildpack ../../target/buildpack/debug/stencila_python
           --path ../../fixtures/projects/python/pyproject-toml/ python-pyproject
```

And if the build succeeds run the container and check that everything was installed and environment variables set properly,

```sh
docker run --rm -it --entrypoint launcher python-pyproject bash
```

It is important to use `--entrypoint launcher` so that environment variables are loaded from the files in `/layers/<layer>/env/` etc

Another option is to test the buildpack thought the "mini cli" of the `buildpacks` internal crate e.g.

```sh
cd rust/buildpacks
cargo run --all-features -- build ../../fixtures/projects/apt/names
```

This has the advantage of being able to easily inspect the files that are installed into the various buildpack layers.

### Testing multiple buildpacks

Sometimes you want to test running multiple buildpacks on a project. Start by making sure that each buildpack is packaged,

```sh
# The `buildpack-%-debug` makefile recipe is just a shortcut for
#   cd rust/buildpack-% && cargo libcnb package
make -C rust/buildpack-python-debug
make -C rust/buildpack-r-debug
```

Then run `pack` with multiple `--buildpack` arguments on a "polyglot" fixture,

```sh
pack build --buildpack target/buildpack/debug/stencila_python \
           --buildpack target/buildpack/debug/stencila_r \
           --path fixtures/projects/polyglot/small \
           polyglot-small
```

### Building a builder

All the Stencila buildpacks are bundled into [builder](https://buildpacks.io/docs/operator-guide/create-a-builder/) images for use in production. To test your buildpack in this context, add it to the list `builder.toml` file and create the builder e.g.

```sh
make -C docker/builder/bionic
```

Then run `pack` using the new builder image,

```sh
pack build --builder stencila/builder:bionic \
           --path fixtures/projects/polyglot/small \
           polyglot-small
```
