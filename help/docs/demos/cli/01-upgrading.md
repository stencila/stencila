---
sidebar_label: Upgrading
title: Upgrade the Stencila CLI
keywords:
  - upgrading
  - Stencila
  - CLI
description: |
  See how to upgrade the Stencila CLI to the latest version, or a specific version, and which configuration settings to adjust for automatic
  upgrades.
---

## Demo

:::info
This demo shows you how to upgrade the Stencila CLI to the latest version, or a specific version, and which configuration settings to adjust for automatic upgrades.
:::

<img src="/docs/demos/cli/01-upgrading.gif" />

## Script

:::tip Run this demo
The above recording was automatically generated from the following Markdown. Copy the code blocks to run the demo yourself.
:::

Let's get started by checking out the help for the `upgrade` command:

```bash pause=2
stencila upgrade --help
```

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
