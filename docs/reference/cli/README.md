---
parts:
  - list
  - open
  - close
  - show
  - run
  - convert
  - diff
  - merge
  - with
  - documents
  - projects
  - sources
  - tasks
  - orgs
  - teams
  - users
  - codecs
  - parsers
  - kernels
  - binaries
  - providers
  - images
  - server
  - config
  - login
  - logout
  - tokens
  - upgrade
  - interact
---


<!-- Generated from doc comments in Rust. Do not edit. -->

# `stencila`: Stencila command line tool

## Usage

```sh
stencila [options] <subcommand>
```

Enter interactive mode by using the `interact` command, the `--interact` option with any other command (will be set as 'prefix'), or not supply any command.

## Subcommands

| Name | Description |
| --- | --- |
| [`list`](list.md) | List all open project and documents |
| [`open`](open.md) | Open a project or document using a web browser |
| [`close`](close.md) | Close a project or document |
| [`show`](show.md) | Show a project or document |
| [`run`](run.md) | Run documents, tasks, and/or server |
| [`convert`](convert.md) | Convert between formats |
| [`diff`](diff.md) | Display the structural differences between two documents |
| [`merge`](merge.md) | Merge changes from two or more derived versions of a document |
| [`with`](with.md) | Run commands interactively with a particular project or document |
| [`documents`](documents/README.md) | Manage documents |
| [`projects`](projects/README.md) | Manage projects |
| [`sources`](sources/README.md) | Manage and use project sources |
| [`tasks`](tasks/README.md) | Manage and run project tasks |
| [`orgs`](orgs/README.md) | Manage organizations |
| [`teams`](teams/README.md) | Manage teams |
| [`users`](users/README.md) | Find and invite users |
| [`codecs`](codecs/README.md) | Manage and use conversion codecs |
| [`parsers`](parsers/README.md) | Manage and use language parsers |
| [`kernels`](kernels/README.md) | Manage and use execution kernels |
| [`binaries`](binaries/README.md) | Manage and use helper binaries |
| [`providers`](providers/README.md) | Manage and use source providers |
| [`images`](images/README.md) | Build and distribute container images |
| [`server`](server/README.md) | Manage document server |
| [`config`](config/README.md) | Manage configuration settings |
| [`login`](login.md) | Login to your Stencila account |
| [`logout`](logout.md) | Logout from your Stencila account |
| [`tokens`](tokens/README.md) | Manage personal access tokens |
| [`upgrade`](upgrade.md) | Upgrade to the latest (or other) version |
| [`interact`](interact.md) | Enter interactive mode (if not yet in it) |
| `help` | Print help information |



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