---
title: Routes Configuration
description: Custom routes for serving content
---

# Routes Configuration

Routes are configured under the `[site.routes]` section.

Custom routes for serving content

Routes map URL paths to files, redirects, or spread configurations.
The key is the URL path (or path template for spreads), and the value can be:
- A simple string for the file path: `"/about/" = "README.md"`
- An object for redirects: `"/old/" = { redirect = "/new/", status = 301 }`
- An object for spreads: `"/{region}/" = { file = "report.smd", arguments = { region = ["north", "south"] } }`

Example:
```toml
[site.routes]
"/" = "index.md"
"/about/" = "README.md"
"/old-page/" = { redirect = "/new-page/", status = 301 }
"/{region}/{species}/" = { file = "report.smd", arguments = { region = ["north", "south"], species = ["ABC", "DEF"] } }
```

## Route Types

### File

Serve a file at this path

Path relative to the workspace directory (or `site.root` if configured).

Example in TOML:
```toml
[site.routes]
"/about/" = "README.md"
```

### Redirect

Redirect to another URL

Example in TOML:
```toml
[site.routes]
"/old/" = { redirect = "/new/", status = 301 }
```

### Spread

Spread configuration for multi-variant routes

Example in TOML:
```toml
[site.routes]
"/{region}/{species}/" = { file = "report.smd", arguments = { region = ["north", "south"], species = ["A", "B"] } }
```

## Redirect Properties

### `redirect`

**Type:** `string`

The URL to redirect to

Can be an absolute URL or a relative path.

Examples:
- /new-location/ - Redirect to another path on the same site
- https://example.com - Redirect to an external URL

### `status`

**Type:** `RedirectStatus` (optional)

HTTP status code for the redirect

Determines the type of redirect. Common values:
- 301 - Moved Permanently (permanent redirect)
- 302 - Found (temporary redirect, default)
- 303 - See Other
- 307 - Temporary Redirect
- 308 - Permanent Redirect

If not specified, defaults to 302 (temporary redirect).

## Spread Properties

### `file`

**Type:** `string`

The source file for this spread route

Path relative to the workspace directory (or `site.root` if configured).

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

