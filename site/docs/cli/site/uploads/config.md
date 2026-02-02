---
title: "`stencila site uploads config`"
description: Configure upload settings
---

Configure upload settings

# Usage

```sh
stencila site uploads config [OPTIONS]
```

# Examples

```bash
# Set upload path
stencila site uploads config --path data

# Set allowed file types
stencila site uploads config --allowed-types csv --allowed-types json

# Set max file size (5MB)
stencila site uploads config --max-size 5242880

# Require commit message
stencila site uploads config --require-message

# Allow users to specify custom paths
stencila site uploads config --user-path

# Only show on admin pages
stencila site uploads config --include "admin/**"
```

# Options

| Name                   | Description                                                                     |
| ---------------------- | ------------------------------------------------------------------------------- |
| `--public`             | Allow public (non-team member) access. Possible values: `true`, `false`.        |
| `--no-public`          | Disallow public access. Possible values: `true`, `false`.                       |
| `--anon`               | Allow anonymous (no GitHub auth) submissions. Possible values: `true`, `false`. |
| `--no-anon`            | Disallow anonymous submissions. Possible values: `true`, `false`.               |
| `--path`               | Unified path for visibility and destination.                                    |
| `--target-path`        | Override: explicit target directory for uploads.                                |
| `--allowed-types`      | Allowed file extensions (can be specified multiple times).                      |
| `--max-size`           | Maximum file size in bytes.                                                     |
| `--user-path`          | Allow users to specify custom upload paths. Possible values: `true`, `false`.   |
| `--no-user-path`       | Disallow custom upload paths. Possible values: `true`, `false`.                 |
| `--allow-overwrite`    | Allow overwriting existing files. Possible values: `true`, `false`.             |
| `--no-allow-overwrite` | Disallow overwriting existing files. Possible values: `true`, `false`.          |
| `--require-message`    | Require a description/commit message. Possible values: `true`, `false`.         |
| `--no-require-message` | Don't require a message. Possible values: `true`, `false`.                      |
| `--include`            | Glob patterns for pages to show widget on (can be specified multiple times).    |
| `--exclude`            | Glob patterns for pages to hide widget from (can be specified multiple times).  |

# Note

Configuring upload settings will automatically enable uploads if not already enabled.
Use stencila site uploads off afterward if you want to disable.
