<!-- Generated from doc comments in Rust. Do not edit. -->

# `create`: Create a new personal access token

## Usage

```sh
stencila auth tokens create [options]
```

Use this command to create a token for accessing the Stencila API on your behalf. Store tokens securely.



## Options

| Name | Description |
| --- | --- |
| `--note -n <note>` | A note for the token. This option is useful for remembering why you created a token and whether you can safely delete it in the future. |
| `--expires-in -e <expires-in>` | The number of minutes until the token should expire. Use this option if you want the new token to expire after a certain amount of time. |
| `--tag -t <tag>` | A tag for the token. Tags are used to identify a token created for a specific client or purpose. They avoid the generation of multiple, redundant tokens. You probably do not need to set a tag when manually creating a token. |

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