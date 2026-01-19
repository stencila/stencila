---
title: "`stencila site add`"
description: Add a route
---

Add a route

# Usage

```sh
stencila site add [OPTIONS] <ROUTE> [FILE] [-- <ARGUMENTS>...]
```

# Examples

```bash
# Add a file route
stencila site add / index.md
stencila site add /about/ README.md

# Add a redirect
stencila site add /old/ --redirect /new/
stencila site add /old/ --redirect /new/ --status 301

# Add external redirect
stencila site add /github/ --redirect https://github.com/stencila/stencila

# Add a spread route (generates multiple variants)
stencila site add "/{region}/" report.smd -- region=north,south
stencila site add "/{region}/{year}/" report.smd -- region=north,south year=2024,2025
stencila site add "/{q}-report/" quarterly.smd --spread zip -- q=q1,q2,q3,q4
```

# Arguments

| Name          | Description                                                        |
| ------------- | ------------------------------------------------------------------ |
| `<ROUTE>`     | Route path (e.g., "/", "/about/", "/{region}/report/").            |
| `[FILE]`      | File to serve at this route.                                       |
| `[ARGUMENTS]` | Arguments for spread routes (comma-delimited key=val1,val2 pairs). |

# Options

| Name                | Description                                                                                                                                                                                          |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-r, --redirect`    | Redirect URL (instead of a file).                                                                                                                                                                    |
| `-s, --status`      | HTTP status code for redirect (301, 302, 303, 307, 308).                                                                                                                                             |
| `--spread <SPREAD>` | Spread mode for multi-variant routes (grid or zip). Possible values: `grid` (Cartesian product of all arguments (default)), `zip` (Positional pairing of values (all params must have same length)). |
