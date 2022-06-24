---
parts:
  - available
  - languages
  - running
  - start
  - connect
  - stop
  - show
  - execute
  - tasks
  - queues
  - cancel
  - symbols
  - restart
  - external
  - directories
---


<!-- Generated from doc comments in Rust. Do not edit. -->

# `kernels`: Manage and use execution kernels

## Usage

```sh
stencila kernels [options] <subcommand>
```



## Subcommands

| Name | Description |
| --- | --- |
| [`available`](available.md) | List the kernels that are available on this machine |
| [`languages`](languages.md) | List the languages supported by the kernels available on this machine |
| [`running`](running.md) | List the kernels in a document kernel space |
| [`start`](start.md) | Start a kernel |
| [`connect`](connect.md) | Connect to a running Jupyter kernel |
| [`stop`](stop.md) | Stop a kernel |
| [`show`](show.md) | Show the details of a current kernel |
| [`execute`](execute.md) | Execute code within a document kernel space |
| [`tasks`](tasks.md) | List the code execution tasks in a document kernel space |
| [`queues`](queues.md) | Show the code execution queues in a document kernel space |
| [`cancel`](cancel.md) | Cancel a code execution task, or all tasks, in a document kernel space |
| [`symbols`](symbols.md) | Show the code symbols in a document kernel space |
| [`restart`](restart.md) | Restart one or all of the kernels |
| [`external`](external.md) | List the Jupyter kernels and servers that are currently running on this machine |
| [`directories`](directories.md) | List the directories on this machine that will be searched for Jupyter kernel specs and running kernels |
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