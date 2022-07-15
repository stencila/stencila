<!-- Generated from doc comments in Rust. Do not edit. -->

# `snap`: Take a snapshot of the filesystem

## Usage

```sh
stencila images snap [options] [dir]
```

This command is used create a snapshot of the filesystem that can be used by the `save` command to generate an image layer based on the changes since the snapshot.

Defaults to creating a snapshot of the entire filesystem but a directory can be specified. Creates a `.snap` file next to the directory that is snap shotted (i.e. defaults to `/root.snap`)

Snapshots are usually made within a container or virtual machine and may be slow if run on a large filesystem. To avoid inadvertent snapshots users are asked for confirmation (this can be skipped by using the `--yes` option).


## Arguments

| Name | Description |
| --- | --- |
| `dir` | Path of the directory to snapshot |

## Options

| Name | Description |
| --- | --- |
| `--yes -y` | Do not ask for confirmation. |

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