---
sidebar_label: Upgrading
title: Upgrade the CLI and plugins
---

## Demo

<img src="01-upgrading.gif" />

## Script

:::tip Run this demo
The above recording was automatically generated from the following Markdown. Copy the code blocks to run the demo yourself.
:::

In this demo we'll show you how to use the upgrade command to upgrade the CLI and plugins.

```bash pause=2
stencila upgrade --help
```

When you run the `upgrade` command, `stencila` will check to see if there is newer version available, and download it if there is. Use `--verbose` to get more information and `--confirm` to prevent an automatic upgrade.

> The following code blocks are currently noexec because --confirm is wrongly defaulting to true

```bash noexec pause=1
stencila upgrade ---verbose
```

Instead of upgrading to the latest version, you can upgrade to a specific version, or downgrade to a previous version.

```bash noexec
stencila upgrade --to 0.53.0
```

Add the `--plugins` option to also check for new versions of installed plugins.

```bash noexec
stencila upgrade --plugins
```
