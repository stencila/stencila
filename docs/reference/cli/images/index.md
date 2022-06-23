---
parts:
  - list
  - build
  - pull
  - push
  - remove
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `images`: Build and distribute container images

## Usage

```sh
stencila images [options] <subcommand>
```

This subcommand provides a limited version of the functionality provided by `docker` and `podman` CLI tools. It is not a general purpose container tool. Only those commands needed by Stencila have been implemented.

## Subcommands

| Name               | Description                                |
| ------------------ | ------------------------------------------ |
| [`list`](list)     | List images in the local image store       |
| [`build`](build)   | Build an image                             |
| [`pull`](pull)     | Pull an image from a registry              |
| [`push`](push)     | Push an image to a registry                |
| [`remove`](remove) | Remove an image from the local image store |
| `help`             | Print help information                     |

## Global options

| Name                        | Description                                                                                                                                          |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--help`                    | Print help information.                                                                                                                              |
| `--version`                 | Print version information.                                                                                                                           |
| `--as <format>`             | Format to display output values (if possible).                                                                                                       |
| `--json`                    | Display output values as JSON (alias for `--as json`).                                                                                               |
| `--yaml`                    | Display output values as YAML (alias for `--as yaml`).                                                                                               |
| `--md`                      | Display output values as Markdown if possible (alias for `--as md`).                                                                                 |
| `--interact -i`             | Enter interactive mode (with any command and options as the prefix).                                                                                 |
| `--debug`                   | Print debug level log events and additional diagnostics. Equivalent to setting `--log-level=debug` and `--log-format=detail` and overrides the both. |
| `--log-level <log-level>`   | The minimum log level to print. One of: `trace`, `debug`, `info`, `warn`, `error`, `never`                                                           |
| `--log-format <log-format>` | The format to print log events. One of: `simple`, `detail`, `json`                                                                                   |
