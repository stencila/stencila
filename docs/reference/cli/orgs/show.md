<!-- Generated from doc comments in Rust. Do not edit. -->

# `show`: Show details of a org

## Usage

```sh
stencila orgs show [options]
```

Use this command to get details on a Stencila org such as when it was last updated. By default, this command shows details for the current org. Use the `--org` option to target another org.



## Options

| Name | Description |
| --- | --- |
| `--org -o <org>` | The id of the org. If this option is not supplied, Stencila will use your default organization. |

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