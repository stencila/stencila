---
sidebar_label: Plugins
title: Managing plugins with the Stencila CLI
keywords:
  - plugins
  - Stencila
  - CLI
description: |
  See how you can use the Stencila CLI to install, upgrade, and uninstall Stencila plugins.
---

:::info
Plugins provide a lot of Stencila's functionality. This demo shows you how to use the Stencila CLI to install, upgrade, and uninstall them.
:::

## Demo

import AsciinemaPlayer from '../../src/components/asciinema/player'
import pluginsDemo from './02-plugins.cast'

<AsciinemaPlayer src={pluginsDemo} />

## Script

:::tip Run this demo
The above recording was automatically generated from the following Markdown. Copy the code blocks to run the demo yourself.
:::

### Listing and viewing plugins

Use the `list` subcommand to get a list of available and/or installed plugins,

```bash pause=1
stencila plugins list
```

The first column is the alias of the plugin (or it's name if it has no alias). If the other columns are all empty it means that you haven't yet refreshed the local plugin registry. Let's do that and list them again,

```bash
stencila plugins refresh
```

```bash pause=2
stencila plugins list
```

Now, you should have information in the other columns such as the latest version, a description and how long ago it was refreshed.

To get more details on a specific plugin, use the `show` subcommand e.g.,

```bash
stencila plugins show javascript
```

### Installing plugins

To install a plugin, just pass its alias or name to the `install` subcommand,

```bash
stencila plugins install javascript
```

Or, if you want to install a specific version

```bash
stencila plugins install javascript@1.9.2
```

Or, use a specific installation method (if available),

```bash
stencila plugins install javascript --docker
```

### Upgrading plugins

Upgrade one or more plugins to their latest versions using,

```bash
stencila plugins upgrade javascript
```

Or, to upgrade all installed plugins,

```bash exec
stencila plugins upgrade
```

### Uninstalling plugins

Use the `uninstall` subcommand to remove a plugin e.g.

```bash
stencila plugins uninstall javascript
```

### Linking and unlinking plugins

If you are developing a plugin, it can be useful to link to its local working directory e.g.

```bash noexec
stencila plugins link ../path/to/myplugin
```

When you no longer need the link, use

```bash noexec
stencila plugins unlink myplugin
```

Now you know how to install and manage plugins, check out some of the other demos for what you can do with them!
