<!-- Generated from doc comments in Rust. Do not edit. -->

# `stop`: Stop a kernel

## Usage

```sh
stencila kernels stop [options] <id>
```

Mainly intended for testing that Jupyter kernels are successfully stopped (e.g. cleanup of connection files).

Only kernels that were started by Stencila can be stopped. A kernel that were started externally by a Jupyter server and then connected to will still run but Stencila will clone any connections to it.


## Arguments

| Name | Description |
| --- | --- |
| `id` | The id of the kernel |


## Global options

| Name | Description |
| --- | --- |
| `--help` | Print help information. |
| `--version` | Print version information. |
| `--as <format>` | Format to display output values (if possible). |
| `--json` | Display output values as JSON (alias for `--as json`). |
| `--yaml` | Display output values as YAML (alias for `--as yaml`). |
| `--md` | Display output values as Markdown if possible (alias for `--as md`). |
| `--interact -i` | Enter interactive mode (with any command and options as the prefix). |
| `--debug` | Print debug level log events and additional diagnostics. Equivalent to setting `--log-level=debug` and `--log-format=detail` and overrides the both. |
| `--log-level <log-level>` | The minimum log level to print. One of: `trace`, `debug`, `info`, `warn`, `error`, `never` |
| `--log-format <log-format>` | The format to print log events. One of: `simple`, `detail`, `json` |