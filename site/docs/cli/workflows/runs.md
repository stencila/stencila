---
title: "`stencila workflows runs`"
description: List recent workflow runs
---

List recent workflow runs

Shows the most recent workflow runs from the workspace database, including run ID, workflow name, goal, status, and timing.

# Usage

```sh
stencila workflows runs [OPTIONS]
```

# Examples

```bash
# List the 20 most recent runs
stencila workflows runs

# List the 5 most recent runs
stencila workflows runs -n 5
```

# Options

| Name          | Description                                                 |
| ------------- | ----------------------------------------------------------- |
| `-n, --limit` | Maximum number of runs to show. Default value: `20`.        |
| `--resumable` | Only show resumable runs. Possible values: `true`, `false`. |
