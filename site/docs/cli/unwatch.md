---
title: "`stencila unwatch`"
description: Disable automatic sync for the workspace or a document
---

Disable automatic sync for the workspace or a document

When run without a path, disables workspace-level watching that runs `update.sh` on each git push.

When run with a path, removes the watch from Stencila Cloud for that document, stopping automatic sync with its remote.

# Usage

```sh
stencila unwatch [PATH] [TARGET]
```

# Examples

```bash
# Disable workspace watch
stencila unwatch

# Disable watch for a document
stencila unwatch report.md

# Unwatch a specific remote (if document has multiple)
stencila unwatch report.md gdoc

# Note: Remote linkage is preserved, you can re-enable watch later
stencila unwatch report.md
stencila watch report.md
```

# Arguments

| Name       | Description                                                |
| ---------- | ---------------------------------------------------------- |
| `[PATH]`   | The path to the document to unwatch (optional).            |
| `[TARGET]` | The target remote to unwatch (only used with a file path). |
