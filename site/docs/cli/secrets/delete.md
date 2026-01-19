---
title: "`stencila secrets delete`"
description: Delete a secret previously set using Stencila
---

Delete a secret previously set using Stencila

# Usage

```sh
stencila secrets delete <NAME>
```

# Examples

```bash
# Delete a specific secret
stencila secrets delete OPENAI_API_KEY

# Delete API tokens
stencila secrets delete ANTHROPIC_API_KEY
stencila secrets delete GOOGLE_AI_API_KEY

# Use the remove alias instead
stencila secrets remove GOOGLE_AI_API_KEY
```

# Arguments

| Name     | Description             |
| -------- | ----------------------- |
| `<NAME>` | The name of the secret. |

# Warning

This permanently removes the secret from your system's
keyring. You'll need to set it again if you want to use
it in the future.
