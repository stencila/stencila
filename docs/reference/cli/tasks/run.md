<!-- Generated from doc comments in Rust. Do not edit. -->

# `run`: Run a task in a Taskfile

## Usage

```sh
stencila tasks run [options] [tasks]
```

Use this command to run one of the tasks in a Taskfile.


## Arguments

| Name | Description |
| --- | --- |
| `tasks` | The names and variables of the tasks to run |

## Options

| Name | Description |
| --- | --- |
| `--schedule -s <schedule>` | Run the tasks on a time schedule. |
| `--watch -w <watch>` | Run the tasks when files matching this pattern change. |
| `--ignore <ignore>` | Ignore changes to files matching this pattern. |
| `--delay -d <delay>` | Number of seconds to delay running tasks after file changes. |
| `--taskfile -f <taskfile>` | The Taskfile to use (defaults to the current). |
| `--error-prefix <error-prefix>` | An internal, hidden option used to contextualize error messages when used as a fallback command. |

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