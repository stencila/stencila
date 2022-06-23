<!-- Generated from doc comments in Rust. Do not edit. -->

# `convert`: Convert between formats

## Usage

```sh
stencila convert [options] <input> [output]
```

## Arguments

| Name     | Description                     |
| -------- | ------------------------------- |
| `input`  | The path of the input document  |
| `output` | The path of the output document |

## Options

| Name                 | Description                                                                                                                                                                                   |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--from -f <from>`   | The format of the input (defaults to being inferred from the file extension or content type).                                                                                                 |
| `--to -t <to>`       | The format of the output (defaults to being inferred from the file extension).                                                                                                                |
| `--compact -c`       | Whether to encode in compact form. Some formats (e.g HTML and JSON) can be encoded in either compact or "pretty-printed" (e.g. indented) forms.                                               |
| `--standalone -s`    | Whether to ensure that the encoded document is standalone. Some formats (e.g. Markdown, DOCX) are always standalone. Others can be fragments, or standalone documents (e.g HTML).             |
| `--bundle -b`        | Whether to bundle local media files into the encoded document. Some formats (e.g. DOCX, PDF) always bundle. For HTML, bundling means including media as data URIs rather than links to files. |
| `--theme -e <theme>` | The theme to apply to the encoded document. Only applies to some formats (e.g. HTML, PDF, PNG).                                                                                               |

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
