---
title: "`stencila status`"
description: Get the tracking status of documents
---

Get the tracking status of documents

# Usage

```sh
stencila status [OPTIONS] [FILES]...
```

# Examples

```bash
# Show status of all tracked documents (includes watch details by default)
stencila status

# Show status of specific documents
stencila status document.md report.md

# Output status as JSON
stencila status --as json

# Skip fetching remote status (faster)
stencila status --no-remotes

# Skip fetching watch status (faster)
stencila status --no-watches
```

# Arguments

| Name      | Description                               |
| --------- | ----------------------------------------- |
| `[FILES]` | The paths of the files to get status for. |

# Options

| Name                        | Description                                                                 |
| --------------------------- | --------------------------------------------------------------------------- |
| `-a, --as <AS>`             | Output the status as JSON or YAML. Possible values: `json`, `yaml`, `toml`. |
| `--no-remotes <NO_REMOTES>` | Skip fetching remote status. Possible values: `true`, `false`.              |
| `--no-watches <NO_WATCHES>` | Skip fetching watch status. Possible values: `true`, `false`.               |
