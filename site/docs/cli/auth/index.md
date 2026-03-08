---
title: "`stencila auth`"
description: Manage OAuth authentication for AI model providers
---

Manage OAuth authentication for AI model providers

# Usage

```sh
stencila auth [COMMAND]
```

# Examples

```bash
# Check which providers you are logged in to
stencila auth

# Login to Anthropic via OAuth
stencila auth login anthropic

# Login to GitHub Copilot
stencila auth login copilot

# Logout from a provider
stencila auth logout gemini
```

# Subcommands

| Command               | Description                                                        |
| --------------------- | ------------------------------------------------------------------ |
| [`status`](status.md) | Display OAuth authentication status for all providers              |
| [`login`](login.md)   | Login to an AI model provider via OAuth                            |
| [`logout`](logout.md) | Logout from an AI model provider (remove stored OAuth credentials) |
