---
title: "`stencila site access password`"
description: Manage password protection
---

Manage password protection

# Usage

```sh
stencila site access password [OPTIONS]
```

# Examples

```bash
# Set a password for the site
stencila site access password

# Clear the password
stencila site access password --clear

# Set password but exclude main/master branches
stencila site access password --not-main
```

# Options

| Name                    | Description                                                                                    |
| ----------------------- | ---------------------------------------------------------------------------------------------- |
| `--clear <CLEAR>`       | Clear the password. Possible values: `true`, `false`.                                          |
| `--not-main <NOT_MAIN>` | Do not apply password protection to main or master branches. Possible values: `true`, `false`. |
| `--main <MAIN>`         | Apply password protection to main or master branches. Possible values: `true`, `false`.        |
