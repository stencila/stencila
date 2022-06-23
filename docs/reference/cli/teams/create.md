<!-- Generated from doc comments in Rust. Do not edit. -->

# `create`: Create a team

## Usage

```sh
stencila teams create [options] <name> [description]
```

Use this command to create a new Stencila team for an organization that you are an admin or owner of.

Defaults to using you default organization. Use the `--org` option to create a team in another organization.

## Arguments

| Name          | Description                |
| ------------- | -------------------------- |
| `name`        | The name of the team       |
| `description` | A description for the team |

## Options

| Name             | Description                                                                                     |
| ---------------- | ----------------------------------------------------------------------------------------------- |
| `--org -o <org>` | The id of the org. If this option is not supplied, Stencila will use your default organization. |

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
