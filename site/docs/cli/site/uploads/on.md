---
title: "`stencila site uploads on`"
description: Enable uploads
---

Enable uploads

# Usage

```sh
stencila site uploads on [OPTIONS]
```

# Examples

```bash
# Enable uploads with default settings
stencila site uploads on

# Enable uploads for data directory
stencila site uploads on --path data

# Enable with allowed file types
stencila site uploads on --allowed-types csv --allowed-types json

# Enable but require authentication
stencila site uploads on --no-public --no-anon
```

# Options

| Name              | Description                                                                     |
| ----------------- | ------------------------------------------------------------------------------- |
| `--path`          | Unified path for visibility and destination.                                    |
| `--allowed-types` | Allowed file extensions (can be specified multiple times).                      |
| `--public`        | Allow public (non-team member) access. Possible values: `true`, `false`.        |
| `--no-public`     | Disallow public access. Possible values: `true`, `false`.                       |
| `--anon`          | Allow anonymous (no GitHub auth) submissions. Possible values: `true`, `false`. |
| `--no-anon`       | Disallow anonymous submissions. Possible values: `true`, `false`.               |
