<!-- Generated from doc comments in Rust. Do not edit. -->

# `query`: Query a document

## Usage

```sh
stencila documents query [options] <file> <query>
```

## Arguments

| Name    | Description                      |
| ------- | -------------------------------- |
| `file`  | The path of the document file    |
| `query` | The query to run on the document |

## Options

| Name                   | Description                                                                 |
| ---------------------- | --------------------------------------------------------------------------- |
| `--format -f <format>` | The format of the file.                                                     |
| `--lang -l <lang>`     | The language of the query. One of: `jmespath`, `jsonptr`. Default: jmespath |

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
