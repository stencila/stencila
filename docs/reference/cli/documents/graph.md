<!-- Generated from doc comments in Rust. Do not edit. -->

# `graph`: Output the dependency graph for a document

## Usage

```sh
stencila documents graph [options] <path>
```

Tip: When using the DOT format (the default), if you have GraphViz and ImageMagick
installed you can view the graph by piping the output to them. For example, to
view a graph of the current project:

```sh
$ stencila documents graph | dot -Tpng | display
```


## Arguments

| Name | Description |
| --- | --- |
| `path` | The path of the document file |

## Options

| Name | Description |
| --- | --- |
| `--format -f <format>` | The format of the document file. |
| `--to -t <to>` | The format to output the graph as. One of: `dot`, `json`, `yaml`. Default: dot |

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