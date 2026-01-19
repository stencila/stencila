---
title: "`stencila track`"
description: Start tracking a document
---

Start tracking a document

# Usage

```sh
stencila track <FILE>
```

# Examples

```bash
# Start tracking a local document
stencila track document.md

# Track multiple documents
stencila track *.md
```

# Arguments

| Name     | Description                          |
| -------- | ------------------------------------ |
| `<FILE>` | The path to the local file to track. |

# Note

Tracking enables version control and change detection for documents.
Configure remotes in stencila.toml for synchronization with external systems.
