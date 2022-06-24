<!-- Generated from doc comments in Rust. Do not edit. -->

# `invite`: Invite users by email or link

## Usage

```sh
stencila users invite [options] <email>
```

Use this command when you want to invite someone who is not yet a Stencila user to join your project or organization.

Stencila will send an invitation message to the user and generate a link that you can also personally send to them (this is advisable if you are concerned about spam filters catching the email).


## Arguments

| Name | Description |
| --- | --- |
| `email` | The email address of the user you wish to invite |

## Options

| Name | Description |
| --- | --- |
| `--no-send` | Do not send an email, just generate a link. Use this option if you do not want Stencila to send an email and will send the invitee the link yourself. |

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