---
title: "`stencila cloud`"
description: Manage Stencila Cloud account
---

Manage Stencila Cloud account

# Usage

```sh
stencila cloud [COMMAND]
```

# Examples

```bash
# Check your cloud authentication status
stencila cloud status

# Sign in to Stencila Cloud
stencila cloud signin

# Sign out from Stencila Cloud
stencila cloud signout

# View logs from a cloud workspace session
stencila cloud logs --session SESSION_ID
```

# Subcommands

| Command                 | Description                                         |
| ----------------------- | --------------------------------------------------- |
| [`status`](status.md)   | Display Stencila Cloud authentication status        |
| [`signin`](signin.md)   | Sign in to Stencila Cloud                           |
| [`signout`](signout.md) | Sign out from Stencila Cloud                        |
| [`logs`](logs.md)       | Display logs from Stencila Cloud workspace sessions |
