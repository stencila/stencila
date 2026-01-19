---
title: "`stencila site domain`"
description: Manage custom domain for the workspace site
---

Manage custom domain for the workspace site

# Usage

```sh
stencila site domain <COMMAND>
```

# Examples

```bash
# Set a custom domain for the site
stencila site domain set example.com

# Check domain status
stencila site domain status

# Remove the custom domain
stencila site domain clear
```

# Subcommands

| Command               | Description                            |
| --------------------- | -------------------------------------- |
| [`set`](set.md)       | Set a custom domain for the site       |
| [`status`](status.md) | Check the status of the custom domain  |
| [`clear`](clear.md)   | Remove the custom domain from the site |
