---
title: "`stencila skills list`"
description: List available skills
---

List available skills

Shows skills from all source directories (`.stencila/skills/`, `.claude/skills/`, `.codex/skills/`, `.gemini/skills/`). Use `--source` to filter by source.

# Usage

```sh
stencila skills list [OPTIONS]
```

# Examples

```bash
# List all skills in table format
stencila skills list

# Output skills as JSON
stencila skills list --as json

# List only skills from .claude/skills/
stencila skills list --source claude
```

# Options

| Name           | Description                                                               |
| -------------- | ------------------------------------------------------------------------- |
| `-a, --as`     | Output the list as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
| `-s, --source` | Filter by source (may be repeated).                                       |

**Possible values of `--source`**

| Value      | Description                                           |
| ---------- | ----------------------------------------------------- |
| `stencila` | `.stencila/skills/` — base layer, always loaded first |
| `claude`   | `.claude/skills/` — Anthropic provider                |
| `codex`    | `.codex/skills/` — OpenAI provider                    |
| `gemini`   | `.gemini/skills/` — Google Gemini provider            |
