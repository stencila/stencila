---
title: "`stencila themes`"
description: Manage themes
---

Manage themes

# Usage

```sh
stencila themes [COMMAND]
```

# Examples

```bash
# List all available themes
stencila themes

# Show the default resolved theme
stencila themes show

# Show a specific theme
stencila themes show tufte

# Create a new workspace theme
stencila themes new

# Create a named user theme
stencila themes new my-theme

# Remove a user theme
stencila themes remove my-theme
```

# Subcommands

| Command               | Description                 |
| --------------------- | --------------------------- |
| [`list`](list.md)     | List the available themes   |
| [`show`](show.md)     | Show the resolved theme CSS |
| [`new`](new.md)       | Create a new theme          |
| [`remove`](remove.md) | Remove a user theme         |
