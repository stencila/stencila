<!-- Generated from doc comments in Rust. Do not edit. -->

# `login`: Login to your Stencila account

## Usage

```sh
stencila login [options]
```

Use this command to link the Stencila CLI to your Stencila account. A browser window will be opened allowing you to sign in to Stencila, or create a Stencila account if you do not have one already. Once you have done that an access token will be stored on your machine allowing to access the Stencila API without having to sign in again.




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