---
title: Outputs Config
description: Outputs configuration.
---

Outputs configuration.

Defines files to be rendered/converted and uploaded to Stencila Cloud
workspace outputs. The key is the output path template, and the value can be:
- A simple source path: `"report.pdf" = "report.md"`
- A configuration object: `"report.pdf" = { source = "report.md", command = "render" }`
- A static file: `"data.csv" = {}` (copies file as-is)
- A pattern: `"exports/*.csv" = { pattern = "exports/*.csv" }`
- A spread: `"{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }`

```toml
# Outputs with source, static, and spread variants
[outputs]
"report.pdf" = "report.md"
"data/results.csv" = {}
"{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
```

## OutputTarget Configuration Entry

Target for an output - either a simple source path or a full configuration

Outputs define files to be rendered/converted and uploaded to Stencila Cloud
workspace outputs. The key is the output path template, e.g.

```toml
# Output mappings for rendered and static files
[outputs]
# Simple: source path (rendered if extension differs)
"report.pdf" = "report.md"

# Full config with options
"report.docx" = { source = "report.md", command = "render" }

# Static file (omit source = use key as source)
"data/results.csv" = {}

# Pattern for multiple files
"exports/*.csv" = { pattern = "exports/*.csv" }

# Spread with parameters
"{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
```

### Simple source path (rendered if extension differs from key)

```toml
[outputs]
"report.pdf" = "report.md"
```

### Full configuration object

```toml
[outputs]
"report.pdf" = { source = "report.md", command = "render" }
"data.csv" = {} # (static, source = output path)
```


***

This documentation was generated from [`outputs.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/outputs.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
