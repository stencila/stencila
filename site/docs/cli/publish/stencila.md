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

| Name                        | Description                                                           |
| --------------------------- | --------------------------------------------------------------------- |
| `-k, --key`                 | The key for the site.                                                 |
| `--dry-run <DRY_RUN>`       | Perform a dry run only. Possible values: `true`, `false`.             |
| `--no-html <NO_HTML>`       | Do not publish a HTML file. Possible values: `true`, `false`.         |
| `--no-jsonld <NO_JSONLD>`   | Do not publish a JSON-LD file. Possible values: `true`, `false`.      |
| `--no-llmd <NO_LLMD>`       | Do not publish a LLM-Markdown file. Possible values: `true`, `false`. |
| `--no-bots <NO_BOTS>`       | Disallow all bots. Possible values: `true`, `false`.                  |
| `--no-ai-bots <NO_AI_BOTS>` | Disallow AI bots. Possible values: `true`, `false`.                   |
