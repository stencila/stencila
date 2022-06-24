<!-- Generated from doc comments in Rust. Do not edit. -->

# `upgrade`: Upgrade to the latest (or other) version

## Usage

```sh
stencila upgrade [options]
```





## Options

| Name | Description |
| --- | --- |
| `--to -t <to>` | Version to upgrade (or downgrade) to (defaults to the latest). |
| `--plugins -p` | Plugins should also be upgraded to their latest version. |
| `--confirm -c` | The user should be asked to confirm an upgrade. |
| `--verbose -v` | Print information on the upgrade process. |

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