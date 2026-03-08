---
title: "`stencila agents validate`"
description: Validate an agent
---

Validate an agent

Checks that an agent conforms to naming and property constraint rules. Accepts an agent name, a directory path, or a path to an AGENT.md file.

# Usage

```sh
stencila agents validate <TARGET>
```

# Examples

```bash
# Validate an agent by name
stencila agents validate code-review

# Validate an agent directory
stencila agents validate .stencila/agents/code-review

# Validate an AGENT.md file directly
stencila agents validate .stencila/agents/code-review/AGENT.md
```

# Arguments

| Name       | Description                                   |
| ---------- | --------------------------------------------- |
| `<TARGET>` | Agent name, directory path, or AGENT.md path. |
