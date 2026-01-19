---
title: "`stencila open`"
description: Open a document in the browser
---

Open a document in the browser

Opens a document in the browser. If the path supplied is a folder then the first file with name `index.*`, `main.*`, or `readme.*` will be opened.

By default, opens both a local preview server and any tracked remote URLs (e.g., Google Docs, Microsoft 365). Use the `target` argument to open only a specific remote (by service shorthand like "gdoc" or "m365", or by full URL).

# Usage

```sh
stencila open [PATH] [TARGET]
```

# Examples

```bash
# Open a specific document (all remotes + local)
stencila open document.md

# Open current directory (finds index/main/readme)
stencila open

# Open only Google Docs remote
stencila open document.md gdoc

# Open only Microsoft 365 remote
stencila open document.md m365

# Open a specific remote URL
stencila open document.md https://docs.google.com/document/d/abc123
```

# Arguments

| Name       | Description                                                    |
| ---------- | -------------------------------------------------------------- |
| `[PATH]`   | The path to the document or parent folder. Default value: `.`. |
| `[TARGET]` | The target to open.                                            |
