---
parts:
  - list
  - show
  - detect
  - plan
  - build
  - pack
  - clean
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `buildpacks`: Manage and use container buildpacks

## Usage

```sh
stencila buildpacks [options] <subcommand>
```

In Stencila, a "buildpack" is a Cloud Native Buildpack (https://buildpacks.io) that is responsible for adding support for a programming language or other type of application to a container image.

## Subcommands

| Name               | Description                                                                             |
| ------------------ | --------------------------------------------------------------------------------------- |
| [`list`](list)     | List the buildpacks available                                                           |
| [`show`](show)     | Show the specifications of a buildpack                                                  |
| [`detect`](detect) | Detect whether a buildpack should build the working directory                           |
| [`plan`](plan)     | Show the build plan for a working directory                                             |
| [`build`](build)   | Build image layers for the working directory using a buildpack                          |
| [`pack`](pack)     | Create a container image for a working directory                                        |
| [`clean`](clean)   | Remove buildpack related directories from the `.stencila` folder or a working directory |
| `help`             | Print help information                                                                  |

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
