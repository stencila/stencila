<!-- Generated from doc comments in Rust. Do not edit. -->

# `diff`: Display the structural differences between two documents

## Usage

```sh
stencila documents diff [options] <first> <second>
```




## Arguments

| Name | Description |
| --- | --- |
| `first` | The path of the first document |
| `second` | The path of the second document |

## Options

| Name | Description |
| --- | --- |
| `--format -f <format>` | The format to display the difference in. Defaults to a "unified diff" of the JSON representation of the documents. Unified diffs of other formats are available e.g. "md", "yaml". Use "raw" for the raw patch as a list of operations. Default: json |

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