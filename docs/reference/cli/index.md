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
  - orgs
  - teams
  - users
  - codecs
  - parsers
  - kernels
  - binaries
  - providers
  - buildpacks
  - images
  - server
  - config
  - login
  - logout
  - tokens
  - upgrade
---

<!-- Generated from doc comments in Rust. Do not edit. -->

# `stencila`: Stencila, in a terminal console, on your own machine

## Usage

```sh
stencila [options] <subcommand>
```

Enter interactive mode by using the `--interact` option with any command.

## Subcommands

| Name                       | Description                                                      |
| -------------------------- | ---------------------------------------------------------------- |
| [`list`](list)             | List all open project and documents                              |
| [`open`](open)             | Open a project or document using a web browser                   |
| [`close`](close)           | Close a project or document                                      |
| [`show`](show)             | Show a project or document                                       |
| [`run`](run)               | Run a document                                                   |
| [`convert`](convert)       | Convert between formats                                          |
| [`diff`](diff)             | Display the structural differences between two documents         |
| [`merge`](merge)           | Merge changes from two or more derived versions of a document    |
| [`with`](with)             | Run commands interactively with a particular project or document |
| [`documents`](documents)   | Manage documents                                                 |
| [`projects`](projects)     | Manage projects                                                  |
| [`sources`](sources)       | Manage and use project sources                                   |
| [`orgs`](orgs)             | Manage organizations                                             |
| [`teams`](teams)           | Manage teams                                                     |
| [`users`](users)           | Find and invite users                                            |
| [`codecs`](codecs)         | Manage and use conversion codecs                                 |
| [`parsers`](parsers)       | Manage and use language parsers                                  |
| [`kernels`](kernels)       | Manage and use execution kernels                                 |
| [`binaries`](binaries)     | Manage and use helper binaries                                   |
| [`providers`](providers)   | Manage and use source providers                                  |
| [`buildpacks`](buildpacks) | Manage and use container buildpacks                              |
| [`images`](images)         | Build and distribute container images                            |
| [`server`](server)         | Manage document server                                           |
| [`config`](config)         | Manage configuration settings                                    |
| [`login`](login)           | Login to your Stencila account                                   |
| [`logout`](logout)         | Logout from your Stencila account                                |
| [`tokens`](tokens)         | Manage personal access tokens                                    |
| [`upgrade`](upgrade)       | Upgrade to the latest (or other) version                         |
| `help`                     | Print help information                                           |

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
