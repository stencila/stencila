<!-- Generated from doc comments in Rust. Do not edit. -->

# `detect`: Detect whether a buildpack should build the working directory

## Usage

```sh
stencila buildpacks detect [options] [working] [label] [platform] [plan]
```

This command is designed to be able to be used in a Cloud Native Buildpack (CNB)
`bin/detect` script e.g

```bash
#!/usr/bin/env bash
set -eo pipefail
stencila buildpacks detect . python $CNB_PLATFORM_DIR $CNB_BUILD_PLAN_PATH
```

See https://github.com/buildpacks/spec/blob/main/buildpack.md#detection
further details.


## Arguments

| Name | Description |
| --- | --- |
| `working` | The working directory (defaults to the current directory) |
| `label` | The id or label of the buildpack to detect with |
| `platform` | A directory containing platform provided configuration, such as environment variables |
| `plan` | A path to a file containing the Build Plan |

## Options

| Name | Description |
| --- | --- |
| `--cnb` | Simulate detection on a CNB platform such as Pack. |

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