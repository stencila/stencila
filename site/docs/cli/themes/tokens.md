---
title: "`stencila themes tokens`"
description: List builtin theme tokens
---

List builtin theme tokens

# Usage

```sh
stencila themes tokens [OPTIONS]
```

# Examples

```bash
# List all builtin theme tokens
stencila themes tokens list

# List tokens for a family
stencila themes tokens list --family admonition

# List tokens for a scope
stencila themes tokens list --scope site

# Output JSON for scripts and agents
stencila themes tokens list --family plot --as json
```

# Options

| Name       | Description                                                                          |
| ---------- | ------------------------------------------------------------------------------------ |
| `--scope`  | Filter by token scope. Possible values: `semantic`, `node`, `site`, `plot`, `print`. |
| `--family` | Filter by token family.                                                              |
| `--as`     | Output as a machine-readable format. Possible values: `json`, `yaml`, `toml`.        |
