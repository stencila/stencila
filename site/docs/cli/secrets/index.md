---
title: "`stencila secrets`"
description: Manage secrets
---

Manage secrets

# Usage

```sh
stencila secrets [COMMAND]
```

# Examples

```bash
# List all configured secrets
stencila secrets

# Set a secret interactively (prompts for value)
stencila secrets set STENCILA_API_TOKEN

# Set a secret from stdin (pipe the value)
echo "sk-abc123..." | stencila secrets set OPENAI_API_KEY

# Delete a secret
stencila secrets delete ANTHROPIC_API_KEY

# Use the add/remove aliases instead
stencila secrets add STENCILA_API_TOKEN
stencila secrets remove STENCILA_API_TOKEN

Security
Secrets are stored securely using your system's keyring.
They are used to authenticate with external services like
AI model providers and cloud platforms.
```

# Subcommands

| Command               | Description                                   |
| --------------------- | --------------------------------------------- |
| [`list`](list.md)     | List the secrets used by Stencila             |
| [`set`](set.md)       | Set a secret used by Stencila                 |
| [`delete`](delete.md) | Delete a secret previously set using Stencila |
