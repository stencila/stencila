---
title: "`stencila formats`"
description: List and inspect supported formats
---

List and inspect supported formats

# Usage

```sh
stencila formats [COMMAND]
```

# Examples

```bash
# List all supported formats
stencila formats list

# Output formats as JSON
stencila formats list --as json

Format Support
• From: Whether the format can be read/imported
• To: Whether the format can be written/exported
• Lossless: Whether conversion preserves all data
```

# Subcommands

| Command                         | Description                                                                          |
| ------------------------------- | ------------------------------------------------------------------------------------ |
| [`list`](list.md)               | List the support for formats                                                         |
| [`structuring`](structuring.md) | Get a list of all structuring operations, or those that are the default for a format |
