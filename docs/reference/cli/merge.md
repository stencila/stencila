<!-- Generated from doc comments in Rust. Do not edit. -->

# `merge`: Merge changes from two or more derived versions of a document

## Usage

```sh
stencila merge [options] <original> <derived>
```

This command can be used as a Git custom "merge driver".
First, register Stencila as a merge driver,

```sh
$ git config merge.stencila.driver "stencila merge --git %O %A %B"
```

(The placeholders `%A` etc are used by `git` to pass arguments such
as file paths and options to `stencila`.)

Then, in your `.gitattributes` file assign the driver to specific
types of files e.g.,

```text
*.{md|docx} merge=stencila
```

This can be done per project, or globally.


## Arguments

| Name | Description |
| --- | --- |
| `original` | The path of the original version |
| `derived` | The paths of the derived versions |

## Options

| Name | Description |
| --- | --- |
| `--git -g` | A flag to indicate that the command is being used as a Git merge driver. When the `merge` command is used as a Git merge driver the second path supplied is the file that is written to. |

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