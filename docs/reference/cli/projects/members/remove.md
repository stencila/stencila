<!-- Generated from doc comments in Rust. Do not edit. -->

# `remove`: Remove a user or team membership of a project

## Usage

```sh
stencila projects members remove [options] <id>
```




## Arguments

| Name | Description |
| --- | --- |
| `id` | The id of the membership |

## Options

| Name | Description |
| --- | --- |
| `--project -p <project>` | The id of the project. If this option is not supplied, Stencila will use the current project. The current project is determined by searching upwards, from the current directory, for a `stencila.yaml`, `stencila.toml`, or `stencila.json` file. |

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