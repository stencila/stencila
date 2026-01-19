---
title: "`stencila site branch delete`"
description: Delete a branch from the site
---

Delete a branch from the site

# Usage

```sh
stencila site branch delete [OPTIONS] <BRANCH_NAME>
```

# Examples

```bash
# Delete a feature branch (with confirmation)
stencila site branch delete feature-xyz

# Delete a branch without confirmation
stencila site branch delete feature-xyz --force

# Delete a branch from another workspace
stencila site branch delete feature-xyz --path /path/to/workspace
```

# Arguments

| Name            | Description                |
| --------------- | -------------------------- |
| `<BRANCH_NAME>` | The branch name to delete. |

# Options

| Name                  | Description                                                     |
| --------------------- | --------------------------------------------------------------- |
| `-p, --path`          | Path to the workspace directory containing .stencila/site.yaml. |
| `-f, --force <FORCE>` | Skip confirmation prompt. Possible values: `true`, `false`.     |

# Notes

- Protected branches (main, master) cannot be deleted
- Deletion is asynchronous and happens in the background
- Cache will be purged automatically for the deleted branch
