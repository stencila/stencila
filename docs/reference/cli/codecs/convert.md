<!-- Generated from doc comments in Rust. Do not edit. -->

# `convert`: Convert between formats

## Usage

```sh
stencila codecs convert [options] <input> [output]
```




## Arguments

| Name | Description |
| --- | --- |
| `input` | The path of the input document |
| `output` | The path of the output document |

## Options

| Name | Description |
| --- | --- |
| `--from -f <from>` | The format of the input (defaults to being inferred from the file extension or content type). |
| `--to -t <to>` | The format of the output (defaults to being inferred from the file extension). |
| `--no-pull` | Do not pull from the remote document for the input (if applicable to the format). |
| `--no-push` | Do not push to the remote document for the output (if applicable to the format). |
| `--compact -c` | Whether to encode in compact form. Some formats (e.g HTML and JSON) can be encoded in either compact or "pretty-printed" (e.g. indented) forms. |
| `--standalone -s` | Whether to ensure that the encoded document is standalone. Some formats (e.g. Markdown, DOCX) are always standalone. Others can be fragments, or standalone documents (e.g HTML). |
| `--bundle -b` | Whether to bundle local media files into the encoded document. Some formats (e.g. DOCX, PDF) always bundle. For HTML, bundling means including media as data URIs rather than links to files. |
| `--theme -e <theme>` | The theme to apply to the encoded document. Only applies to some formats (e.g. HTML, PDF, PNG). |
| `--lossy -l` | Whether to convert to the target format with loss. This option disables Stencila's extensions to make formats such as DOCX and PDF reproducible. |
| `--rpng-types <rpng-types>` | The document node types (e.g `CodeChunk`, `MathFragment`) to encode as ReproduciblePNGs. . If no types are provided, a standard set of types will be used for the particular format. |
| `--rpng-text` | Whether to store the JSON representation of a document node as the alt text of a RPNG image. May always be enabled if the format requires it for reproducibility. |
| `--rpng-link` | Whether to surround RPNGs in a link to the JSON representation of the document node on Stencila Cloud. . May always be enabled if the format requires it for reproducibility. |

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