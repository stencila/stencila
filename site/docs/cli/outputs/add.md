---
title: "`stencila outputs add`"
description: Add an output configuration
---

Add an output configuration

# Usage

```sh
stencila outputs add [OPTIONS] <OUTPUT> [SOURCE] [-- <ARGUMENTS>...]
```

# Examples

```bash
# Add a simple output (render report.md to report.pdf)
stencila outputs add report.pdf report.md

# Add with explicit render command
stencila outputs add report.pdf report.md --command render

# Add output that only runs on main branch
stencila outputs add report.pdf report.md --refs main

# Add static file (copy as-is)
stencila outputs add data.csv

# Add pattern-based outputs
stencila outputs add "exports/*.pdf" --pattern "exports/*.md"

# Add spread output (generates multiple variants)
stencila outputs add "{region}/report.pdf" report.md --command render -- region=north,south

# Add spread with multiple arguments (grid mode)
stencila outputs add "{region}/{year}/data.pdf" report.md --command render -- region=north,south year=2024,2025

# Add spread with zip mode
stencila outputs add "{q}-report.pdf" report.md --command render --spread zip -- q=q1,q2,q3,q4
```

# Arguments

| Name          | Description                                                         |
| ------------- | ------------------------------------------------------------------- |
| `<OUTPUT>`    | Output path.                                                        |
| `[SOURCE]`    | Source file path.                                                   |
| `[ARGUMENTS]` | Arguments for spread outputs (comma-delimited key=val1,val2 pairs). |

# Options

| Name                      | Description                                                                                                                                                                                                                                      |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `-c, --command <COMMAND>` | Processing command. Possible values: `render` (Execute code and convert to output format (default for different extensions)), `convert` (Format transformation only, no code execution), `none` (Copy file as-is (default for same extensions)). |
| `-r, --refs`              | Git ref patterns for when to process this output.                                                                                                                                                                                                |
| `-p, --pattern`           | Glob pattern for matching multiple source files.                                                                                                                                                                                                 |
| `-e, --exclude`           | Glob patterns to exclude from pattern matches.                                                                                                                                                                                                   |
| `--spread <SPREAD>`       | Spread mode for multi-variant outputs (grid or zip). Possible values: `grid` (Cartesian product of all arguments (default)), `zip` (Positional pairing of values (all params must have same length)).                                            |
