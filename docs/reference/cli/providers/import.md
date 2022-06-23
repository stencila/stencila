<!-- Generated from doc comments in Rust. Do not edit. -->

# `import`: Import content from a remote source to a local path

## Usage

```sh
stencila providers import [options] <source> [path]
```

## Arguments

| Name     | Description                                         |
| -------- | --------------------------------------------------- |
| `source` | The source identifier e.g. `github:org/name@v1.2.0` |
| `path`   | The local path to import file/s to e.g. `data`      |

## Options

| Name              | Description                                                                                                                                                                                                                                               |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
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
