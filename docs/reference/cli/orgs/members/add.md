<!-- Generated from doc comments in Rust. Do not edit. -->

# `add`: Add a user as a member of a organization

## Usage

```sh
stencila orgs members add [options] <user> [role]
```

Use this command to a add a user to a organization. When you add a team to a organization, all the users that are members of that team get the same role on the organization. The default role is "member". Specify "owner" or "admin" roles for greater permissions on the organization.


## Arguments

| Name | Description |
| --- | --- |
| `user` | The username or id of the user |
| `role` | The role to give the user |

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