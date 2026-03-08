---
title: "`stencila auth login`"
description: Login to an AI model provider via OAuth
---

Login to an AI model provider via OAuth

# Usage

```sh
stencila auth login <PROVIDER>
```

# Examples

```bash
# Login to Anthropic
stencila auth login anthropic

# Login to GitHub Copilot
stencila auth login copilot

# Login to Google Gemini
stencila auth login gemini

# Login to OpenAI
stencila auth login openai
```

# Arguments

| Name         | Description                                                                            |
| ------------ | -------------------------------------------------------------------------------------- |
| `<PROVIDER>` | The provider to login to. Possible values: `anthropic`, `copilot`, `gemini`, `openai`. |
