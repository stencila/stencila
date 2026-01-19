---
title: "`stencila site domain status`"
description: Check the status of the custom domain
---

Check the status of the custom domain

# Usage

```sh
stencila site domain status [OPTIONS]
```

# Examples

```bash
# Check domain status
stencila site domain status

# Check status for another workspace
stencila site domain status --path /path/to/workspace
```

# Options

| Name         | Description                                                     |
| ------------ | --------------------------------------------------------------- |
| `-p, --path` | Path to the workspace directory containing .stencila/site.yaml. |
