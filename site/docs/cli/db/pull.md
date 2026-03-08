---
title: "`stencila db pull`"
description: Pull database state from Stencila Cloud
---

Pull database state from Stencila Cloud

# Usage

```sh
stencila db pull [OPTIONS]
```

# Examples

```bash
# Pull latest database state
stencila db pull

# Force pull when local state has diverged
stencila db pull --force
```

# Options

| Name      | Description                                                                                           |
| --------- | ----------------------------------------------------------------------------------------------------- |
| `--force` | Force pull even when local database has diverged from the manifest. Possible values: `true`, `false`. |
