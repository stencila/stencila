---
title: "`stencila linters list`"
description: List the linters available
---

List the linters available

# Usage

```sh
stencila linters list [OPTIONS]
```

# Examples

```bash
# List all available linters
stencila linters list

# List only Python linters
stencila linters list --lang py

# List only citation linters
stencila linters list --node-type Citation

# Output linter list as YAML
stencila linters list --as yaml
```

# Options

| Name              | Description                                                               |
| ----------------- | ------------------------------------------------------------------------- |
| `-l, --language`  | Only list linter that support a specific language/format.                 |
| `-n, --node-type` | Only list linter that support a specific node type.                       |
| `-a, --as`        | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
