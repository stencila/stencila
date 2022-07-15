<!-- Generated from doc comments in Rust. Do not edit. -->

# `list`: List tasks in a Taskfile

## Usage

```sh
stencila tasks list [options]
```

Use this command to quickly get a list of all the tasks in a Taskfile.



## Options

| Name | Description |
| --- | --- |
| `--all -a` | List all tasks, including those in included Taskfiles. By default only task that are defined in the root Taskfile are listed. Use this option to show all tasks, including those from included Taskfiles. |
| `--topic -t <topic>` | Filter tasks by topic e.g. 'python', 'git'. |
| `--action -c <action>` | Filter tasks by action e.g. 'add', 'remove'. |
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