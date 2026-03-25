---
title: "`stencila auth logout`"
description: Logout from an AI model provider (remove stored OAuth credentials)
---

Logout from an AI model provider (remove stored OAuth credentials)

# Usage

```sh
stencila auth logout <PROVIDER>
```

# Examples

```bash
# Logout from Anthropic
stencila auth logout anthropic

# Logout from GitHub Copilot
stencila auth logout copilot
```

# Arguments

| Name         | Description                                                                                |
| ------------ | ------------------------------------------------------------------------------------------ |
| `<PROVIDER>` | The provider to logout from. Possible values: `anthropic`, `copilot`, `gemini`, `open-ai`. |
