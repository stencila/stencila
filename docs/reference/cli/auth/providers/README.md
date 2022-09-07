---
parts:
  - list
  - connect
  - disconnect
  - token
---


<!-- Generated from doc comments in Rust. Do not edit. -->

# `providers`: Manage authentication providers

## Usage

```sh
stencila auth providers [options] <subcommand>
```



## Subcommands

| Name | Description |
| --- | --- |
| [`list`](list.md) | List external accounts connected to your Stencila account |
| [`connect`](connect.md) | Connect an external account to your Stencila account |
| [`disconnect`](disconnect.md) | Disconnect an external account from your Stencila account |
| [`token`](token.md) | Obtain an access token for a provider |
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