<!-- Generated from doc comments in Rust. Do not edit. -->

# `build`: Build image layers for the working directory using a buildpack

## Usage

```sh
stencila buildpacks build [options] [working] [label] [layers] [platform] [build]
```

This command is designed to be able to be used in a Cloud Native Buildpack (CNB)
`bin/build` script e.g

```bash
#!/usr/bin/env bash
set -eo pipefail
stencila buildpacks build . python $CNB_LAYERS_DIR $CNB_PLATFORM_DIR $CNB_BP_PLAN_PATH
```

See https://github.com/buildpacks/spec/blob/main/buildpack.md#build for
further details.

## Arguments

| Name       | Description                                                                                                                     |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `working`  | The working directory (defaults to the current directory)                                                                       |
| `label`    | The id or label of the buildpack to build                                                                                       |
| `layers`   | A directory that will contain subdirectories representing each layer created by the buildpack in the final image or build cache |
| `platform` | A directory containing platform provided configuration, such as environment variables                                           |
| `build`    | A path to a file containing the Buildpack Plan                                                                                  |

## Options

| Name    | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--cnb` | Simulate building on a CNB platform such as Pack. This is useful to buildpack developers for local debugging. For example, in another terminal, run `watch tree ...` on a project,. watch tree -a -L 6 fixtures/projects/node/package-json/. and then run build that project with the `--cnb` flag,. cargo run --bin stencila -- buildpacks build --cnb fixtures/projects/node/package-json/. Equivalent to using `/tmp/cnb` as `platform` directory (so won't work on platforms without `/tmp`). |

## Global options

| Name                        | Description                                                                                                                                          |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--help`                    | Print help information.                                                                                                                              |
| `--version`                 | Print version information.                                                                                                                           |
| `--as <format>`             | Format to display output values (if possible).                                                                                                       |
| `--json`                    | Display output values as JSON (alias for `--as json`).                                                                                               |
| `--yaml`                    | Display output values as YAML (alias for `--as yaml`).                                                                                               |
| `--md`                      | Display output values as Markdown if possible (alias for `--as md`).                                                                                 |
| `--interact -i`             | Enter interactive mode (with any command and options as the prefix).                                                                                 |
| `--debug`                   | Print debug level log events and additional diagnostics. Equivalent to setting `--log-level=debug` and `--log-format=detail` and overrides the both. |
| `--log-level <log-level>`   | The minimum log level to print. One of: `trace`, `debug`, `info`, `warn`, `error`, `never`                                                           |
| `--log-format <log-format>` | The format to print log events. One of: `simple`, `detail`, `json`                                                                                   |
