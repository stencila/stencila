---
title: "`stencila workflows discard`"
description: Discard an ephemeral workflow
---

Discard an ephemeral workflow

Removes an ephemeral workflow directory that was created by an agent. Only ephemeral workflows can be discarded; permanent workflows must be deleted manually.

# Usage

```sh
stencila workflows discard <NAME>
```

# Examples

```bash
# Discard an ephemeral workflow
stencila workflows discard my-workflow
```

# Arguments

| Name     | Description                          |
| -------- | ------------------------------------ |
| `<NAME>` | The name of the workflow to discard. |
