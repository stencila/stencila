---
parts:
  - login
  - logout
  - me
  - find
  - invite
---


<!-- Generated from doc comments in Rust. Do not edit. -->

# `users`: Find and invite users

## Usage

```sh
stencila users [options] <subcommand>
```



## Subcommands

| Name | Description |
| --- | --- |
| [`login`](login.md) | Login to your Stencila account |
| [`logout`](logout.md) | Logout from your Stencila account |
| [`me`](me.md) | Show the currently authenticated user |
| [`find`](find.md) | Find users by name |
| [`invite`](invite.md) | Invite users by email or link |
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