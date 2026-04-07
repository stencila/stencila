---
title: "`stencila config show`"
description: Show the current configuration
---

Show the current configuration

# Usage

```sh
stencila config show [OPTIONS]
```

# Examples

```bash
# Show entire configuration
stencila config show

# Show as JSON
stencila config show --as json

# Show as TOML
stencila config show --as toml
```

# Options

| Name       | Description                                                                                  |
| ---------- | -------------------------------------------------------------------------------------------- |
| `-a, --as` | Output format (toml, json, or yaml, default: yaml). Possible values: `json`, `yaml`, `toml`. |
