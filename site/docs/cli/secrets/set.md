---
title: "`stencila secrets set`"
description: Set a secret used by Stencila
---

Set a secret used by Stencila

You will be prompted for the secret. Alternatively, you can echo the password into this command i.e. `echo <TOKEN> | stencila secrets set STENCILA_API_TOKEN`

# Usage

```sh
stencila secrets set <NAME>
```

# Examples

```bash
# Set a secret interactively (you'll be prompted)
stencila secrets set OPENAI_API_KEY

# Set a secret from stdin
echo "sk-abc123..." | stencila secrets set OPENAI_API_KEY

# Set API tokens for different services
stencila secrets set ANTHROPIC_API_KEY
stencila secrets set GOOGLE_AI_API_KEY
stencila secrets set STENCILA_API_TOKEN

# Use the add alias instead
stencila secrets add STENCILA_API_TOKEN

Security
When setting secrets interactively, your input will be
hidden. When piping from stdin, ensure your shell history
doesn't record the command with the secret value.
```

# Arguments

| Name     | Description             |
| -------- | ----------------------- |
| `<NAME>` | The name of the secret. |
