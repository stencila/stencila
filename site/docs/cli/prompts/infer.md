---
title: "`stencila prompts infer`"
description: Infer a prompt from a query
---

Infer a prompt from a query

Useful for checking which prompt will be matched to a given instruction type, node types, and/or query

# Usage

```sh
stencila prompts infer [OPTIONS] [QUERY]
```

# Examples

```bash
# Infer prompt with a specific query
stencila prompts infer "Update this paragraph based on latest data"

# Infer for a specific instruction type
stencila prompts infer --instruction-type create "list of top regions"
```

# Arguments

| Name      | Description |
| --------- | ----------- |
| `[QUERY]` | The query.  |

# Options

| Name                     | Description           |
| ------------------------ | --------------------- |
| `-i, --instruction-type` | The instruction type. |
| `-n, --node-types`       | The node types.       |
