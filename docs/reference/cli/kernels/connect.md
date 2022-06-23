<!-- Generated from doc comments in Rust. Do not edit. -->

# `connect`: Connect to a running Jupyter kernel

## Usage

```sh
stencila kernels connect [options] <id-or-path>
```

Mainly intended for testing that Stencila is able to connect
to an existing kernel (e.g. one that was started from Jupyter notebook).

To get a list of externally started kernels that can be connected to run,

```stencila
> kernels external
```

and then connect to a kernel using its Jupyter id e.g.,

```stencila
> kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
```

Alternatively, use the path (relative or absolute) of the Jupyter notebook
whose (already started) kernel you wish to connect to e.g.,

```stencila
> kernels connect ../main.ipynb
```

## Arguments

| Name         | Description                                                                                           |
| ------------ | ----------------------------------------------------------------------------------------------------- |
| `id-or-path` | The id of the kernel e.g. `31248fc2-38d0-4d11-80a1-f8a1bd3842fb` or the relative path of the notebook |

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
