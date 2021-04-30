---
sidebar_label: Upgrading
title: Upgrade the Stencila CLI
keywords:
  - upgrading
  - Stencila
  - CLI
description: |
  See how to upgrade the Stencila CLI to the latest version, or a specific version, and which configuration settings to adjust for automatic upgrades.
---

import AsciinemaPlayer from '../../../src/components/asciinema/player'
import upgradingDemo from './01-upgrading.cast'

:::info
This demo shows you how to upgrade the Stencila CLI to the latest version, or a specific version, and which configuration settings to adjust for automatic upgrades.
:::

## Demo

<AsciinemaPlayer src={upgradingDemo} />

## Script

:::tip Run this demo
The above recording was automatically generated from the following Markdown. Copy the code blocks to run the demo yourself.
:::

Let's get started by checking out the help for the `upgrade` command:

```bash pause=2
stencila upgrade --help
```

### Manual upgrades

When you run the `upgrade` command, `stencila` will check to see if there is newer version available, and download it if there is. Use `--verbose` to get more information and `--confirm` to prevent an automatic upgrade.

```bash pause=1
stencila upgrade ---verbose
```

Instead of upgrading to the latest version, you can upgrade to a specific version, or downgrade to a previous version.

```bash pause=1
stencila upgrade --to 0.56.0
```

Add the `--plugins` option to also check for new versions of installed plugins.

```bash pause=1
stencila upgrade --plugins
```

### Automatic upgrades

Stencila can be configured to do automatic upgrades. Each time the `stencila` CLI tool is run it will check for a newer version (if it has not already checked within a configured duration, by default, one day). If a newer version is available, by default, you will be asked to confirm the upgrade.

See your current settings using,

```bash pause=2
stencila config get upgrade
```

To turn off automatic upgrades,

```bash
stencila config set upgrade.auto off
```

To make them less frequent,

```bash
stencila config set upgrade.auto '1 week'
```

To allow upgrades to happen without your confirmation,

```bash
stencila config set upgrade.confirm false
```

To not upgrade plugins as well,

```bash
stencila config set upgrade.plugins false
```

To reset them to defaults,

```bash
stencila config reset upgrade
```
