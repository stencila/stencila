<!-- Generated from doc comments in Rust. Do not edit. -->

# `execute`: Execute code within a document kernel space

## Usage

```sh
stencila documents execute [options] <path> [code]
```

Mainly intended for testing that Stencila is able to talk
to Jupyter kernels and execute code within them.

Use the `--kernel` option to specify, by name, language or type, which kernel the code
should be executed in e.g.,

```stencila
> kernels execute Math.PI --lang=javascript
```

```stencila
> kernels execute Math.PI --lang javascript --kernel="type:jupyter"
```

In interactive mode, you can set the command prefix to "stay" in a particular
language and mimic a REPL in that language e.g.,

```stencila
> kernels execute --lang=javascript
> let r = 10
> 2 * Math.PI * r
```

If a kernel is not yet running for the language then one will be started
(if installed on the machine).


## Arguments

| Name | Description |
| --- | --- |
| `path` | The path of the document file |
| `code` | Code to execute within the kernel space |

## Options

| Name | Description |
| --- | --- |
| `--format -f <format>` | The format of the document file. |
| `--lang -l <lang>` | The programming language of the code. |
| `--kernel -k <kernel>` | The kernel where the code should executed (a kernel selector string). |
| `--background -b` | The task should run be in the background. |
| `--fork` | The task should run be in a kernel fork (if possible). |

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