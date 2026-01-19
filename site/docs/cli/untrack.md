---
title: "`stencila untrack`"
description: Stop tracking a document
---

Stop tracking a document

# Usage

```sh
stencila untrack <FILE>
```

# Examples

```bash
# Stop tracking a document
stencila untrack document.md

# Stop tracking all tracked files
stencila untrack all
```

# Arguments

| Name     | Description                            |
| -------- | -------------------------------------- |
| `<FILE>` | The path of the file to stop tracking. |

# Note

This removes the document from tracking but does not
delete the file itself.
