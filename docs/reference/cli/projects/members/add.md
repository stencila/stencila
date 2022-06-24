<!-- Generated from doc comments in Rust. Do not edit. -->

# `add`: Add a user or team as a member of a project

## Usage

```sh
stencila projects members add [options] <type> <id> [role]
```

Use this command to a add a user or team to a project. When you add a team to a project, all the users that are members of that team get the same role on the project. The default role is "member". Specify "owner" or "admin" roles for greater permissions on the project.


## Arguments

| Name | Description |
| --- | --- |
| `type` | The type of member to add |
| `id` | The id of the user or team |
| `role` | The role to give the user or team |

## Options

| Name | Description |
| --- | --- |
| `--project -p <project>` | The id of the project. If this option is not supplied, Stencila will use the current project. The current project is determined by searching upwards, from the current directory, for a `stencila.toml`, `stencila.yaml`, or `stencila.json` file. |

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