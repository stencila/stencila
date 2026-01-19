---
title: "`stencila site preview`"
description: Preview the workspace site locally with live reload
---

Preview the workspace site locally with live reload

# Usage

```sh
stencila site preview [OPTIONS] [ROUTE]
```

# Examples

```bash
# Preview site at root
stencila site preview

# Preview a specific route
stencila site preview /docs/guide/

# Preview without opening browser
stencila site preview --no-open

# Preview on different port
stencila site preview --port 8080

# Preview without file watching
stencila site preview --no-watch
```

# Arguments

| Name      | Description                                                |
| --------- | ---------------------------------------------------------- |
| `[ROUTE]` | Route to open in browser (default: /). Default value: `/`. |

# Options

| Name                    | Description                                                          |
| ----------------------- | -------------------------------------------------------------------- |
| `-p, --port <PORT>`     | Port to serve on. Default value: `9000`.                             |
| `--no-open <NO_OPEN>`   | Do not open browser automatically. Possible values: `true`, `false`. |
| `--no-watch <NO_WATCH>` | Do not watch for file changes. Possible values: `true`, `false`.     |
