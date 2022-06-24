<!-- Generated from doc comments in Rust. Do not edit. -->

# `list`: List projects

## Usage

```sh
stencila projects list [options]
```

Use this command to get a list of Stencila projects that you are a member of, or that are public. For more details on a particular project use the `show` sibling command.

Use the optional search string to filter projects using the name and title properties of projects.

By default, only shows projects that you are a member of, use the `--all` flag to include projects that you are not a member of but which are public. Use the `--role` flag to only include projects for which you have a particular role.



## Options

| Name | Description |
| --- | --- |
| `--search -s <search>` | A search string to filter projects by. |
| `--role -r <role>` | Only list projects for which you have a specific role. The role may be granted directly to you, or via a team. One of: `owner`, `admin`, `member` |
| `--org -o <org>` | Only list projects which belong to a particular organization. Use a numeric id or organization's short name. |
| `--all -a` | List all projects, including public project that you are not a member of. To avoid getting a long list of projects, you generally only want to use this flag in conjunction with a search string. |

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