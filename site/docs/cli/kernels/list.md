---
title: "`stencila kernels list`"
description: List the kernels available
---

List the kernels available

# Usage

```sh
stencila kernels list [OPTIONS]
```

# Examples

```bash
# List all available kernels
stencila kernels list

# List only math kernels
stencila kernels list --type math

# Output kernel list as YAML
stencila kernels list --as yaml
```

# Options

| Name                | Description                                                                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-t, --type <TYPE>` | Only list kernels of a particular type. Possible values: `programming`, `database`, `templating`, `diagrams`, `visualization`, `math`, `styling`. |
| `-a, --as <AS>`     | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`.                                                                         |
