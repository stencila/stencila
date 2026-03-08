---
title: "`stencila workflows validate`"
description: Validate a workflow
---

Validate a workflow

Checks that a workflow conforms to naming and property constraint rules, and validates the pipeline DOT if present. Accepts a workflow name, a directory path, or a path to a WORKFLOW.md file.

# Usage

```sh
stencila workflows validate <TARGET>
```

# Examples

```bash
# Validate a workflow by name
stencila workflows validate data-pipeline

# Validate a workflow directory
stencila workflows validate .stencila/workflows/data-pipeline

# Validate a WORKFLOW.md file directly
stencila workflows validate .stencila/workflows/data-pipeline/WORKFLOW.md
```

# Arguments

| Name       | Description                                         |
| ---------- | --------------------------------------------------- |
| `<TARGET>` | Workflow name, directory path, or WORKFLOW.md path. |
