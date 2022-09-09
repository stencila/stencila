<!-- Generated from doc comments in Rust. Do not edit. -->

# `disconnect`: Disconnect an external account from your Stencila account

## Usage

```sh
stencila auth providers disconnect [options] <provider>
```




## Arguments

| Name | Description |
| --- | --- |
| `provider` | The name of the authentication provider |

## Options

| Name | Description |
| --- | --- |
| `--web -w` | Open the corresponding web page on Stencila in your browser. Use this option when you want to quickly jump to the web page on Stencila that offers the same, or similar, functionality to this command. |

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