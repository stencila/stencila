<!-- Generated from doc comments in Rust. Do not edit. -->

# `list`: List organizations

## Usage

```sh
stencila orgs list [options]
```

Use this command to get a list of Stencila organizations. For more details on a particular organization use the `show` sibling command.

Use the optional search string to filter organizations using their names (short and long).

By default, only shows organizations that you are a member of, use the `--all` option to include all organizations. Use the `--role` option to only include organizations for which you have a particular role.

## Options

| Name                   | Description                                                                                                                                                                                               |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--search -s <search>` | A search string to filter organizations by.                                                                                                                                                               |
| `--role -r <role>`     | Only list organizations for which you have a specific role. The role may be granted directly to you, or via a team. One of: `owner`, `admin`, `member`                                                    |
| `--all -a`             | List all organizations, including organization that you are not a member of. To avoid getting a long list of organizations, you generally only want to use this flag in conjunction with a search string. |

## Global options

| Name                        | Description                                                                                                                                          |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--help`                    | Print help information.                                                                                                                              |
| `--version`                 | Print version information.                                                                                                                           |
| `--as <format>`             | Format to display output values (if possible).                                                                                                       |
| `--json`                    | Display output values as JSON (alias for `--as json`).                                                                                               |
| `--yaml`                    | Display output values as YAML (alias for `--as yaml`).                                                                                               |
| `--md`                      | Display output values as Markdown if possible (alias for `--as md`).                                                                                 |
| `--interact -i`             | Enter interactive mode (with any command and options as the prefix).                                                                                 |
| `--debug`                   | Print debug level log events and additional diagnostics. Equivalent to setting `--log-level=debug` and `--log-format=detail` and overrides the both. |
| `--log-level <log-level>`   | The minimum log level to print. One of: `trace`, `debug`, `info`, `warn`, `error`, `never`                                                           |
| `--log-format <log-format>` | The format to print log events. One of: `simple`, `detail`, `json`                                                                                   |
