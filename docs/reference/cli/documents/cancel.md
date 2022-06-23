<!-- Generated from doc comments in Rust. Do not edit. -->

# `cancel`: Cancel a code execution task, or all tasks, in a document kernel space

## Usage

```sh
stencila documents cancel [options] <path> <task>
```

Use an integer to cancel a task by it's number. Use "all" to cancel all unfinished tasks.

## Arguments

| Name   | Description                     |
| ------ | ------------------------------- |
| `path` | The path of the document file   |
| `task` | The task number or id, or "all" |

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
