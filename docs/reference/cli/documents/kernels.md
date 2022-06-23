<!-- Generated from doc comments in Rust. Do not edit. -->

# `kernels`: List the kernels in a document kernel space

## Usage

```sh
stencila documents kernels [options] <path>
```

Mainly intended for interactive mode testing / inspection. Note that
for a kernel to be in this list it must have either been started by Stencila,

```stencila
> kernels start r
```

or connected to from Stencila,

```stencila
> kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
```

To get a list of externally started Jupyter kernels that can be connected to run,

```stencila
> kernels external
```

## Arguments

| Name   | Description                   |
| ------ | ----------------------------- |
| `path` | The path of the document file |

## Options

| Name                   | Description                      |
| ---------------------- | -------------------------------- |
| `--format -f <format>` | The format of the document file. |

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
