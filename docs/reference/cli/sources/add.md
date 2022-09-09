<!-- Generated from doc comments in Rust. Do not edit. -->

# `add`: Add a source to a project

## Usage

```sh
stencila sources add [options] <url> [dest] [project]
```

Does not import the source use the `import` command for that.


## Arguments

| Name | Description |
| --- | --- |
| `url` | The URL (or "short URL" e.g github:owner/repo@v1.1) of the source to be added |
| `dest` | The path to import the source to |
| `project` | The project to add the source to (defaults to the current project) |

## Options

| Name | Description |
| --- | --- |
| `--name -n <name>` | The name to give the source. |
| `--cron -c <cron>` | A cron schedule for the source. |
| `--watch -w <watch>` | A watch mode for the source. |
| `--dry-run` | Do a dry run of adding the source. Parses the input URL and other arguments into a source but does not add it, or the files that it imports, to the project. Useful for checking URL and cron formats and previewing the files that will be imported. |

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