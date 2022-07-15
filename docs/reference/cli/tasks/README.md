---
parts:
  - init
  - list
  - run
  - detect
  - update
  - docs
---


<!-- Generated from doc comments in Rust. Do not edit. -->

# `tasks`: Manage and run project tasks

## Usage

```sh
stencila tasks [options] <subcommand>
```



## Subcommands

| Name | Description |
| --- | --- |
| [`init`](init.md) | Initialize a tasks for a directory |
| [`list`](list.md) | List tasks in a Taskfile |
| [`run`](run.md) | Run a task in a Taskfile |
| [`detect`](detect.md) | Detect dependencies and tasks for a project |
| [`update`](update.md) | Update a Taskfile to include detected tasks |
| [`docs`](docs.md) | Generate docs for Taskfiles |
| `help` | Print help information |



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