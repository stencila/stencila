---
title: "`stencila site reviews on`"
description: Enable reviews
---

Enable reviews

# Usage

```sh
stencila site reviews on [OPTIONS]
```

# Examples

```bash
# Enable reviews with default settings
stencila site reviews on

# Enable reviews and allow public submissions
stencila site reviews on --public

# Enable reviews but require GitHub authentication
stencila site reviews on --no-anon
```

# Options

| Name                      | Description                                                                     |
| ------------------------- | ------------------------------------------------------------------------------- |
| `--public <PUBLIC>`       | Allow public (non-team member) submissions. Possible values: `true`, `false`.   |
| `--no-public <NO_PUBLIC>` | Disallow public submissions. Possible values: `true`, `false`.                  |
| `--anon <ANON>`           | Allow anonymous (no GitHub auth) submissions. Possible values: `true`, `false`. |
| `--no-anon <NO_ANON>`     | Disallow anonymous submissions. Possible values: `true`, `false`.               |
