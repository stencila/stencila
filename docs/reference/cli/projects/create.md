<!-- Generated from doc comments in Rust. Do not edit. -->

# `create`: Create a project

## Usage

```sh
stencila projects create [options]
```

Use this command to create a new Stencila project. A new project will be created on Stencila Cloud and a `stencila.toml` file will be created, with the new project's id, in the current folder.

Use the `--org` option to select the organization for the project.

## Options

| Name                 | Description                                                                                                                    |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `--name -n <name>`   | The name of the project. Must be unique within the organization. Defaults to a randomly generated name.                        |
| `--title -t <title>` | The title of the project.                                                                                                      |
| `--public -p`        | Whether the project should be public. New projects default to being private. Use the this flag to make the new project public. |
| `--org -o <org>`     | The organization under which to create the project. Use the organization's numeric id.                                         |

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
