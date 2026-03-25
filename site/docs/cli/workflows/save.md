---
title: "`stencila workflows save`"
description: Save an ephemeral workflow
---

Save an ephemeral workflow

Removes the ephemeral marker from a workflow that was created by an agent, converting it into a permanent workspace workflow.

# Usage

```sh
stencila workflows save <NAME>
```

# Examples

```bash
# Save an ephemeral workflow
stencila workflows save my-workflow
```

# Arguments

| Name     | Description                       |
| -------- | --------------------------------- |
| `<NAME>` | The name of the workflow to save. |
