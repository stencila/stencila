---
title: "`stencila site access team`"
description: Manage team access restriction
---

Manage team access restriction

# Usage

```sh
stencila site access team [OPTIONS]
```

# Examples

```bash
# Enable team access restriction
stencila site access team

# Disable team access restriction
stencila site access team --off

# Enable but exclude main/master branches
stencila site access team --not-main
```

# Options

| Name         | Description                                                                            |
| ------------ | -------------------------------------------------------------------------------------- |
| `--off`      | Disable team access restriction. Possible values: `true`, `false`.                     |
| `--not-main` | Do not apply restriction to main or master branches. Possible values: `true`, `false`. |
| `--main`     | Apply restriction to main or master branches. Possible values: `true`, `false`.        |
