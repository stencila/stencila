---
title: "`stencila publish stencila`"
description: Publish to Stencila Cloud
---

Publish to Stencila Cloud

# Usage

```sh
stencila publish stencila [OPTIONS] [PATH]
```

# Arguments

| Name     | Description                                                   |
| -------- | ------------------------------------------------------------- |
| `[PATH]` | Path to the file or directory to publish. Default value: `.`. |

# Options

| Name           | Description                                                           |
| -------------- | --------------------------------------------------------------------- |
| `-k, --key`    | The key for the site.                                                 |
| `--dry-run`    | Perform a dry run only. Possible values: `true`, `false`.             |
| `--no-html`    | Do not publish a HTML file. Possible values: `true`, `false`.         |
| `--no-jsonld`  | Do not publish a JSON-LD file. Possible values: `true`, `false`.      |
| `--no-llmd`    | Do not publish a LLM-Markdown file. Possible values: `true`, `false`. |
| `--no-bots`    | Disallow all bots. Possible values: `true`, `false`.                  |
| `--no-ai-bots` | Disallow AI bots. Possible values: `true`, `false`.                   |
