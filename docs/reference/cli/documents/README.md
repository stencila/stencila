---
parts:
  - list
  - open
  - close
  - show
  - build
  - graph
  - params
  - run
  - plan
  - query
  - diff
  - merge
  - detect
---


<!-- Generated from doc comments in Rust. Do not edit. -->

# `documents`: Manage documents

## Usage

```sh
stencila documents [options] <subcommand>
```



## Subcommands

| Name | Description |
| --- | --- |
| [`list`](list.md) | List open documents |
| [`open`](open.md) | Open a document |
| [`close`](close.md) | Close a document |
| [`show`](show.md) | Show a document |
| [`build`](build.md) | Build a document |
| [`graph`](graph.md) | Output the dependency graph for a document |
| [`params`](params.md) | Show the parameters of a document |
| [`run`](run.md) | Run a document |
| [`plan`](plan.md) | Generate an execution plan for a document |
| [`query`](query.md) | Query a document |
| [`diff`](diff.md) | Display the structural differences between two documents |
| [`merge`](merge.md) | Merge changes from two or more derived versions of a document |
| [`detect`](detect.md) | Detect entities within a document |
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