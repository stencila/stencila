---
title: "`stencila config`"
description: Manage Stencila configuration
---

Manage Stencila configuration

# Usage

```sh
stencila config [COMMAND]
```

# Examples

```bash
# Show the current configuration
stencila config

# Show configuration as JSON
stencila config get --as json

# Get a specific config value
stencila config get site.id

# Set a value in the nearest stencila.toml
stencila config set site.id mysite123

# Set a value in user config
stencila config set --user site.id mysite123

# Set a value in local override file
stencila config set --local site.id mysite123

# Remove a value
stencila config unset site.id

# Check config validity
stencila config check
```

# Subcommands

| Command             | Description                  |
| ------------------- | ---------------------------- |
| [`get`](get.md)     | Get configuration value(s)   |
| [`set`](set.md)     | Set a configuration value    |
| [`unset`](unset.md) | Remove a configuration value |
| [`check`](check.md) | Check configuration validity |
