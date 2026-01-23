---
title: "`stencila move`"
description: Move a tracked document
---

Move a tracked document

Moves the document file to the new path (if it still exists at the old path) and updates any tracking information.

# Usage

```sh
stencila move [OPTIONS] <FROM> <TO>
```

# Examples

```bash
# Move a tracked document
stencila move old-name.md new-name.md

# Move to a different directory
stencila move document.md docs/document.md

# Force overwrite if destination exists
stencila move source.md target.md --force

# Use the mv alias
stencila mv old.md new.md
```

# Arguments

| Name     | Description               |
| -------- | ------------------------- |
| `<FROM>` | The old path of the file. |
| `<TO>`   | The new path of the file. |

# Options

| Name          | Description                                                                            |
| ------------- | -------------------------------------------------------------------------------------- |
| `-f, --force` | Overwrite the destination path if it already exists. Possible values: `true`, `false`. |

# Note

This updates both the file system and tracking
information. If the destination already exists,
you'll be prompted unless --force is used.
