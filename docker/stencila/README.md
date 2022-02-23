# 🐳 `stencila/stencila`

**A Docker image designed for using Stencila at the command line**

## Purpose

This image is intended primarily for those who would like to use the `stencila` command line tool without installing it locally.

## Installation

```sh
docker pull stencila/stencila
```

## Usage

The image uses `stencila` as it's `ENTRYPOINT` so just pass Stencila commands and options directly after the image name e.g.

```sh
docker run -it --rm stencila/stencila --help
```

The image has `/workspace` as its working directory and a default `stencila` user. You can mount your local directory to `/workspace` and make yourself the user using Docker's `-v` and `-u` options e.g.

```sh
docker run -it --rm -p 9000 -v $PWD:/workspace -u $(id -u):$(id -g) stencila/stencila
```

If you want to access the Stencila server at `http://localhost:9000` make sure to expose port `9000` and serve on all addresses (`0.0.0.0`) from within the container e.g.

```sh
docker run -it --rm -p 9000 stencila/stencila server start --url 0.0.0.0:9000
```

You can combine all these option into a shell alias e.g.

```sh
alias stencila='docker run -it --rm -p 9000 -v $PWD:/workspace -u $(id -u):$(id -g) stencila/stencila'
```
