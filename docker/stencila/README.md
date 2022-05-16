# 🐳 `stencila/stencila`

**A Docker image for using Stencila at the command line**

## Purpose

This image is intended primarily for those who would like to use the `stencila` command line tool without installing it locally. It downloads and installs the latest release of the Stencila CLI from the Stencila GitHub repository.

This image is intentionally minimal. For example, it does not contain helper binaries such as `pandoc` and `chromium` that Stencila delegates to for conversion between some document formats. Nor does it contain runtimes for programming languages such as Python and R. If you need these, we suggest that you install the Stencila CLI locally, or use one of the other, more comprehensive images we provide.

## Installation

```sh
docker pull stencila/stencila
```

## Usage

The image uses `stencila` as it's `ENTRYPOINT` so just pass Stencila commands and options directly after the image name e.g.

```sh
docker run -it --init --rm stencila/stencila --help
```

The image has `/workspace` as its working directory and a default `stencila` user. You can mount your local directory to `/workspace` and make yourself the user using Docker's `-v` and `-u` options e.g.

```sh
docker run -it --init --rm -v $PWD:/workspace -u $(id -u):$(id -g) stencila/stencila
```

If you want to access the Stencila document server at `http://localhost:9000` make sure to expose port `9000` and serve on all addresses (`0.0.0.0`) from within the container e.g.

```sh
docker run -it --init --rm -p 9000:9000 stencila/stencila server start --url 0.0.0.0:9000
```

You can combine all these option into a shell alias e.g.

```sh
alias stencila='docker run -it --init --rm -p 9000:9000 -v $PWD:/workspace -u $(id -u):$(id -g) stencila/stencila'
```
