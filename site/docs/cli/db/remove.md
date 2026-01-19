---
title: "`stencila db remove`"
description: Remove documents from the workspace database
---

Remove documents from the workspace database

# Usage

```sh
stencila db remove <DOCUMENTS>...
```

# Examples

```bash
# Remove a document from workspace database
stencila db remove document.md

# Remove multiple documents
stencila db remove *.md docs/*.md

# Use the rm alias
stencila db rm old-document.md
```

# Arguments

| Name          | Description                                         |
| ------------- | --------------------------------------------------- |
| `<DOCUMENTS>` | The document to remove from the workspace database. |

# Note

This removes documents from the workspace database
but does not delete the actual files. The files
will no longer be indexed or queryable.
