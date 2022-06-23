<!-- Generated from doc comments in Rust. Do not edit. -->

# `parse`: Parse some code using a parser

## Usage

```sh
stencila parsers parse [options] [code]
```

The code is parsed into a set of graph `Relation`/`Resource` pairs using the parser that matches the filename extension (or specified using `--lang`). Useful for testing Stencila's static code analysis for a particular language.

## Arguments

| Name   | Description                 |
| ------ | --------------------------- |
| `code` | The file (or code) to parse |

## Options

| Name               | Description                                                         |
| ------------------ | ------------------------------------------------------------------- |
| `--text -t`        | If the argument should be treated as text, rather than a file path. |
| `--lang -l <lang>` | The language of the code.                                           |

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
