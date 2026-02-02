---
title: "`stencila site reviews config`"
description: Configure review settings
---

Configure review settings

# Usage

```sh
stencila site reviews config [OPTIONS]
```

# Examples

```bash
# Allow public submissions
stencila site reviews config --public

# Disallow anonymous submissions
stencila site reviews config --no-anon

# Only allow comments (not suggestions)
stencila site reviews config --types comment

# Allow both comments and suggestions
stencila site reviews config --types comment --types suggestion

# Set selection limits
stencila site reviews config --min-selection 10 --max-selection 2000

# Enable keyboard shortcuts (Ctrl+Shift+C for comment, Ctrl+Shift+S for suggestion)
stencila site reviews config --shortcuts

# Only show reviews on docs and guides pages
stencila site reviews config --include "docs/**" --include "guides/**"

# Hide reviews from API reference and changelog
stencila site reviews config --exclude "api/**" --exclude "changelog/**"
```

# Options

| Name              | Description                                                                                       |
| ----------------- | ------------------------------------------------------------------------------------------------- |
| `--public`        | Allow public (non-team member) submissions. Possible values: `true`, `false`.                     |
| `--no-public`     | Disallow public submissions. Possible values: `true`, `false`.                                    |
| `--anon`          | Allow anonymous (no GitHub auth) submissions. Possible values: `true`, `false`.                   |
| `--no-anon`       | Disallow anonymous submissions. Possible values: `true`, `false`.                                 |
| `--types`         | Allowed review types (can be specified multiple times). Possible values: `comment`, `suggestion`. |
| `--min-selection` | Minimum selection length in characters.                                                           |
| `--max-selection` | Maximum selection length in characters.                                                           |
| `--shortcuts`     | Enable keyboard shortcuts. Possible values: `true`, `false`.                                      |
| `--no-shortcuts`  | Disable keyboard shortcuts. Possible values: `true`, `false`.                                     |
| `--include`       | Glob patterns for paths to show reviews on (can be specified multiple times).                     |
| `--exclude`       | Glob patterns for paths to hide reviews from (can be specified multiple times).                   |

# Note

Configuring review settings will automatically enable reviews if not already enabled.
Use stencila site reviews off afterward if you want to disable.
