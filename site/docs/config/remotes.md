---
title: Remotes Configuration
description: Remote synchronization configuration
---

# Remotes Configuration

Remote synchronization configuration

Maps local paths to remote service URLs. The key is the local path
(file, directory, or pattern), and the value can be:
- A simple URL string: `"site" = "https://example.stencila.site/"`
- An object with watch: `"file.md" = { url = "...", watch = "w123" }`
- Multiple remotes: `"file.md" = [{ url = "...", watch = "..." }, "https://..."]`

Directory paths are implicitly recursive, matching all files within.

Example:
```toml
[remotes]
"site" = "https://example.stencila.site/"
"docs/report.md" = { url = "https://docs.google.com/...", watch = "w123" }
"article.md" = [
{ url = "https://docs.google.com/...", watch = "w456" },
"https://sharepoint.com/..."
]
```

## Remote Types

### Url

Simple URL string (no watch)

Example: `"https://example.stencila.site/"`

### Watch

URL with watch information

Example: `{ url = "https://...", watch = "w123" }`

### Spread

Spread configuration for multi-variant pushes

Example: `{ service = "gdoc", title = "Report {region}", params = { region = ["north", "south"] } }`

## Watch Properties

### `url`

**Type:** `string (URL)`

Remote URL

The service type is inferred from the URL host:
- Google Docs: https://docs.google.com/document/d/...
- Microsoft 365: https://*.sharepoint.com/...
- Stencila Sites: https://*.stencila.site/...

### `watch`

**Type:** `string` (optional)

**Pattern:** `^w[a-zA-Z0-9]{9}$`

Watch ID from Stencila Cloud

If this remote is being watched for automatic synchronization, this
field contains the watch ID. Watch configuration (direction, PR mode,
debounce) is stored in Stencila Cloud and queried via the API.

## Spread Properties

### `service`

**Type:** `string`

Target service

One of: "gdoc", "m365"

### `title`

**Type:** `string` (optional)

Title template with placeholders

Placeholders like `{param}` are replaced with parameter values.
Example: "Report - {region}"

### `spread`

**Type:** `SpreadMode` (optional)

Spread mode

- `grid`: Cartesian product of all parameter values (default)
- `zip`: Positional pairing of values (all params must have same length)

### `arguments`

**Type:** `HashMap`

Parameter values for spread variants

Keys are parameter names, values are arrays of possible values.
Example: `{ region = ["north", "south"], species = ["A", "B"] }`

