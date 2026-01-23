---
title: "`stencila`"
description: CLI subcommands and global options
---

CLI subcommands and global options

# Usage

```sh
stencila [OPTIONS] <COMMAND>
```

# Examples

```bash
# Get help on all available commands
stencila --help

# Create a new document
stencila new article.md

# Convert a document to another format
stencila convert input.md output.pdf

# Check available formats
stencila formats list

# Execute a document
stencila execute notebook.myst

# Open a document in the browser
stencila open document.md
```

# Subcommands

| Command                       | Description                                                          |
| ----------------------------- | -------------------------------------------------------------------- |
| [`new`](new.md)               | Create a new, tracked, document                                      |
| [`init`](init.md)             | Initialize a workspace with stencila.toml configuration              |
| [`config`](config/index.md)   | Manage Stencila configuration                                        |
| [`status`](status.md)         | Get the tracking status of documents                                 |
| [`move`](move.md)             | Move a tracked document                                              |
| [`track`](track.md)           | Start tracking a document                                            |
| [`untrack`](untrack.md)       | Stop tracking a document                                             |
| [`clean`](clean.md)           | Clean the current workspace                                          |
| [`convert`](convert.md)       | Convert a document to another format                                 |
| [`merge`](merge.md)           | Merge changes from another format                                    |
| [`sync`](sync.md)             | Synchronize a document between formats                               |
| [`push`](push.md)             | Push content to Stencila Cloud and remote services                   |
| [`pull`](pull.md)             | Pull a document from a remote service                                |
| [`watch`](watch.md)           | Enable automatic sync for the workspace or a document                |
| [`unwatch`](unwatch.md)       | Disable automatic sync for the workspace or a document               |
| [`compile`](compile.md)       | Compile a document                                                   |
| [`lint`](lint.md)             | Lint one or more documents                                           |
| [`execute`](execute.md)       | Execute a document                                                   |
| [`render`](render.md)         | Render a document                                                    |
| [`query`](query.md)           | Query a workspace database                                           |
| [`open`](open.md)             | Open a document in the browser                                       |
| [`publish`](publish/index.md) | Publish one or more documents                                        |
| [`demo`](demo.md)             | Run a terminal demonstration from a document                         |
| [`outputs`](outputs/index.md) | Manage workspace outputs                                             |
| [`db`](db/index.md)           | Manage the workspace and other document databases                    |
| [`prompts`](prompts/index.md) | Manage prompts                                                       |
| [`models`](models/index.md)   | Manage and interact with generative AI models                        |
| [`kernels`](kernels/index.md) | Manage execution kernels                                             |
| [`linters`](linters/index.md) | Manage linters                                                       |
| [`formats`](formats/index.md) | List and inspect supported formats                                   |
| [`themes`](themes/index.md)   | Manage themes                                                        |
| [`secrets`](secrets/index.md) | Manage secrets                                                       |
| [`tools`](tools/index.md)     | Manage tools and environments used by Stencila                       |
| [`serve`](serve.md)           | Run the HTTP/Websocket server                                        |
| [`snap`](snap.md)             | Capture screenshots and measurements of documents served by Stencila |
| [`lsp`](lsp.md)               | Run the Language Server Protocol server                              |
| [`cloud`](cloud/index.md)     | Manage Stencila Cloud account                                        |
| [`site`](site/index.md)       | Manage the workspace site                                            |
| [`signin`](signin.md)         | Sign in to Stencila Cloud                                            |
| [`signout`](signout.md)       | Sign out from Stencila Cloud                                         |
| [`logs`](logs.md)             | Display logs from Stencila Cloud workspace sessions                  |
| [`upgrade`](upgrade.md)       | Upgrade to the latest version                                        |
| [`uninstall`](uninstall.md)   | Uninstall this command line tool                                     |

# Options

| Name         | Description                                                                                   |
| ------------ | --------------------------------------------------------------------------------------------- |
| `-h, --help` | Print help: `-h` for brief help, `--help` for more details. Possible values: `true`, `false`. |
| `--yes`      | Assume the answer `yes` to any interactive prompts. Possible values: `true`, `false`.         |
| `--debug`    | Display debug level logging and detailed error reports. Possible values: `true`, `false`.     |
| `--no-color` | Do not color any output.                                                                      |
