<!-- Generated from doc comments in Rust. Do not edit. -->

# `run`: Run a document

## Usage

```sh
stencila run [options] <input> [output]
```




## Arguments

| Name | Description |
| --- | --- |
| `input` | The path of the document to execute |
| `output` | The path to save the executed document |

## Options

| Name | Description |
| --- | --- |
| `--from -f <from>` | The format of the input (defaults to being inferred from the file extension or content type). |
| `--to -t <to>` | The format of the output (defaults to being inferred from the file extension). |
| `--theme -e <theme>` | The theme to apply to the output (only for HTML and PDF). |
| `--start -s <start>` | The id of the node to start execution from. |
| `--ordering -o <ordering>` | Ordering for the execution plan. |
| `--concurrency -c <concurrency>` | Maximum concurrency for the execution plan. A maximum concurrency of 2 means that no more than two tasks will run at the same time (ie. in the same stage). Defaults to the number of CPUs on the machine. |
| `--dry-run -d` | Generate execution plan but do not execute it. |
| `--quiet -q` | Do not display execution plan or progress. |

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