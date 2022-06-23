<!-- Generated from doc comments in Rust. Do not edit. -->

# `enrich`: Enrich nodes within a file or string

## Usage

```sh
stencila providers enrich [options] <path> [format]
```

## Arguments

| Name     | Description                                                               |
| -------- | ------------------------------------------------------------------------- |
| `path`   | The path to the file (or the string value if the `--string` flag is used) |
| `format` | The format of the file; defaults to the file extension                    |

## Options

| Name              | Description                                                                                                                                                                                                                                               |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--string -s`     | If the argument should be treated as a string, rather than a file path.                                                                                                                                                                                   |
| `--token <token>` | The token (or name of environment variable) required to access the resource. Only necessary if authentication is required for the resource. Defaults to using the environment variable corresponding to the provider of the resource e.g. `GITHUB_TOKEN`. |

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
