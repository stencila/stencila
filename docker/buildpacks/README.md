# 📚 Cloud Native Buildpack (CNB) Stacks

**A set of CNB stacks for building and running custom containers using Stencila**

## Purpose

For authors of CNB `buildpacks` who want to use Stencila in builder or application images. 

A CNB [_stack_](https://buildpacks.io/docs/concepts/components/stack/) is composed of two container images that are intended to work together:

- The `build` image of a stack provides the base image from which the build environment is constructed. The build environment is the containerized environment in which `buildpacks` are executed.

- The `run` image of a stack provides the base image from which application images are built.

The `Dockerfile` for each stack in this folder will build both the `build` and `run` image.

## Installation

```sh
docker pull stencila/build
docker pull stencila/run
```

## Usage

When [creating a builder](https://buildpacks.io/docs/operator-guide/create-a-builder/), specify the stack in your `builder.toml`:

```toml
[stack]
id = "stencila.stacks.focal"
run-image = "stencila/run:focal"
build-image = "stencila/build:focal"
```

When creating a buildpack, add the stack to `[[stacks]]` in your `buildpack.toml`:


```toml
[[stacks]]
id = "stencila.stacks.focal"
```
