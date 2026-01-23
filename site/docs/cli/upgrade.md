---
title: "`stencila upgrade`"
description: Upgrade to the latest version
---

Upgrade to the latest version

# Usage

```sh
stencila upgrade [OPTIONS]
```

# Examples

```bash
# Upgrade to the latest version
stencila upgrade

# Check if an upgrade is available without installing
stencila upgrade --check

# Force upgrade even if current version is latest
stencila upgrade --force
```

# Options

| Name          | Description                                                                                  |
| ------------- | -------------------------------------------------------------------------------------------- |
| `-f, --force` | Perform upgrade even if the current version is the latest. Possible values: `true`, `false`. |
| `-c, --check` | Check for an available upgrade but do not install it. Possible values: `true`, `false`.      |

# Note

Upgrade downloads the latest release from GitHub and replaces
the current binary.
