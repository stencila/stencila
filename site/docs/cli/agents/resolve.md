---
title: "`stencila agents resolve`"
description: Show how an agent session would be routed
---

Show how an agent session would be routed

Dry-run routing resolution: shows the provider, model, session type, credential source, and reasoning without starting a session. Useful for debugging why a particular provider or model was chosen.

# Usage

```sh
stencila agents resolve [OPTIONS] <NAME>
```

# Examples

```bash
# Show routing for the default agent
stencila agents resolve default

# Show extended routing details
stencila agents resolve code-engineer --why
```

# Arguments

| Name     | Description                       |
| -------- | --------------------------------- |
| `<NAME>` | The name of the agent to resolve. |

# Options

| Name    | Description                                                      |
| ------- | ---------------------------------------------------------------- |
| `--why` | Show extended routing details. Possible values: `true`, `false`. |
