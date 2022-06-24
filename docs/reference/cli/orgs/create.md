<!-- Generated from doc comments in Rust. Do not edit. -->

# `create`: Create an organization

## Usage

```sh
stencila orgs create [options]
```

Use this command to create a new Stencila organization. Use the `--default` option to make the new organization your default.



## Options

| Name | Description |
| --- | --- |
| `--short-name -s <short-name>` | A "short name" of the organization. Must be unique. Used in URLs for the organization on Stencila Cloud. Defaults to a randomly generated name. |
| `--long-name -l <long-name>` | A "long name" of the organization. Used mainly for display purposes. |
| `--default -d` | Make the new organization your default organization. Use this option to make the new organization your default. It will then be used instead of having to specify the `--org` option in the CLI or on the web. |

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