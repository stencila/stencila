<!-- Generated from Taskfile. Do not edit. -->

# `podman`: Tasks related to `podman`

## Includes

Other `Taskfile`s included:

- [`asdf`](asdf.md)

## Tasks

### <a id='detect'>`detect`</a> : Detect which Podman related tasks are needed

Adds the `podman:build` task if there is a `Dockerfile` in the root of the project.

#### Command

```sh
test -f Dockerfile && echo podman:build >> .stencila/tasks/detected
```

### <a id='install'>`install`</a> : Ensure Podman is installed for the project

Checks whether Podman is installed, and if is not, installs it using `asdf`.

#### Commands

1. [`asdf:ensure-plugin`](asdf.md#ensure-plugin) `NAME=podman` `URL=https://github.com/nokome/asdf-podman`

2. [`asdf:add`](asdf.md#add) `PACKAGE=podman` `VERSION=3.4.4`

### <a id='build'>`build`</a> : Build the `Dockerfile`

#### Sources

- `Dockerfile`

#### Commands

1. [`install`](#install)

2. `podman build .`
