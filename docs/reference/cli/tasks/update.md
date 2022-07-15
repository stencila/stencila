<!-- Generated from doc comments in Rust. Do not edit. -->

# `update`: Update a Taskfile to include detected tasks

## Usage

```sh
stencila tasks update [options]
```

This command is usually only used internally as a "callback" to update the Taskfile at the end of the `detect` task.



## Options

| Name | Description |
| --- | --- |
| `--taskfile -f <taskfile>` | The Taskfile to use (defaults to the current). |

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