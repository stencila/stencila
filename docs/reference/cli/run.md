<!-- Generated from doc comments in Rust. Do not edit. -->

# `run`: Run documents, tasks, and/or server

## Usage

```sh
stencila run [options] [args]
```

Use this command to quickly run one or more documents, tasks or the server.
It provides a short cut to the `documents run`, `tasks run`, and `server run`
subcommands and allows you to chain those together.

## Tasks

Given a `Taskfile.yaml` in the current directory with a task named `simulation`,
the command,

```sh
stencila run simulation n=100
```

is equivalent to `stencila tasks run simulation n=100`.

All tasks in the `Taskfile.yaml` with a `schedule` or `watches` can be
run concurrently using,

```sh
stencila run tasks
```

which is equivalent to `stencila tasks run`.

## Documents

If the current directory does not have a `Taskfile.yaml`, or the argument does not
match a task in the current Taskfile, the argument will be assumed to be a filename.

The command,

```sh
stencila run report.md
```

is equivalent to `stencila documents run report.md`.

## Server

The argument `server` will run the server with default options e.g.

```sh
stencila run server
```

is equivalent to `stencila server run`.

## Backgrounding

Things can be run in the background by adding a tilde `~`. For example, to run a task
and a document concurrently,

```sh
stencila run simulation~ n=100 report.md~
```

## Default

If no arguments are supplied, the default is to run all tasks with a `schedule` or `watches`
in the background (if a `Taskfile.yaml` is present), and to run the server i.e.

```sh
stencila run
```

is equivalent to `stencila run tasks~ server`.


## Arguments

| Name | Description |
| --- | --- |
| `args` | Run arguments |


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