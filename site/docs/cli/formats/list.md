---
title: "`stencila formats list`"
description: List the support for formats
---

List the support for formats

# Usage

```sh
stencila formats list [OPTIONS]
```

# Examples

```bash
# List all supported formats in table format
stencila formats list

# Export format information as JSON
stencila formats list --as json

Columns
• Name: The format name
• Extension: Default file extension
• From: Can read/import this format
• To: Can write/export this format
• Lossless: Whether conversion preserves all data
```

# Options

| Name       | Description                                                               |
| ---------- | ------------------------------------------------------------------------- |
| `-a, --as` | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
