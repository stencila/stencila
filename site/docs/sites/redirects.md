---
title: Site Redirects
description: Configure URL redirects for your site using stencila.toml or per-directory files
---

Stencila sites support two redirect mechanisms. Config redirects defined in `stencila.toml` support pattern matching with placeholders and splats. Per-directory `_redirect.json` files handle simple single-route cases.

# Config redirects

Redirect routes are defined under `[site.routes]` in `stencila.toml`. Placeholders use `{name}` syntax (consistent with [spread routes](/docs/config/site#routes)) and splats use `*`, e.g.

```toml
[site.routes]
# Exact redirect
"/old-page" = { redirect = "/new-page", status = 301 }

# Splat — * matches all remaining path segments
"/blog/*" = { redirect = "https://blog.example.com/{splat}", status = 301 }

# Placeholders — {name} matches a single path segment
"/docs/{lang}/{page}" = { redirect = "/documentation/{lang}/{page}" }

# Combined placeholder and splat
"/products/{category}/*" = { redirect = "/shop/{category}/{splat}", status = 301 }

# External redirect (defaults to 307)
"/legacy/" = { redirect = "https://legacy.example.org" }
```

## Fields

| Field      | Type    | Required | Description                                                                                |
| ---------- | ------- | -------- | ------------------------------------------------------------------------------------------ |
| `redirect` | string  | yes      | Target URL — use `{name}` to substitute placeholders and `{splat}` to substitute the splat |
| `status`   | integer | no       | HTTP status code: `301`, `302`, `303`, `307`, or `308`. Defaults to `307`                  |

## Patterns

### Exact match

```toml
"/old-page" = { redirect = "/new-page", status = 301 }
```

For static sites hosted on Stencila Sites, exact redirects are intended for a single route path. To avoid ambiguity, prefer writing routes with your site's canonical trailing-slash style.

### Splats (`*`)

A `*` at the end of the route path greedily matches all remaining path segments. Use `{splat}` in the target to substitute the matched value.

```toml
"/blog/*" = { redirect = "https://blog.example.com/{splat}", status = 301 }
```

| Request              | `{splat}` value | Redirects to                            |
| -------------------- | --------------- | --------------------------------------- |
| `/blog/`             | _(empty)_       | `https://blog.example.com/`             |
| `/blog/hello`        | `hello`         | `https://blog.example.com/hello`        |
| `/blog/2024/my-post` | `2024/my-post`  | `https://blog.example.com/2024/my-post` |

Only one `*` is allowed per rule, and it must be the last segment.

### Placeholders (`{name}`)

A `{name}` segment matches exactly one path segment. Use `{name}` in the target to substitute the captured value.

```toml
"/docs/{lang}/{page}" = { redirect = "/documentation/{lang}/{page}" }
```

| Request                | Captures                         | Redirects to              |
| ---------------------- | -------------------------------- | ------------------------- |
| `/docs/en/intro`       | `lang=en`, `page=intro`          | `/documentation/en/intro` |
| `/docs/fr/guide`       | `lang=fr`, `page=guide`          | `/documentation/fr/guide` |
| `/docs/en`             | _(no match — too few segments)_  | —                         |
| `/docs/en/intro/extra` | _(no match — too many segments)_ | —                         |

Placeholder names must be alphanumeric (plus underscores) and unique within a rule.

### Combining splats and placeholders

```toml
"/products/{category}/*" = { redirect = "/shop/{category}/{splat}", status = 301 }
```

| Request                        | Redirects to               |
| ------------------------------ | -------------------------- |
| `/products/shoes/`             | `/shop/shoes/`             |
| `/products/shoes/nike/air-max` | `/shop/shoes/nike/air-max` |

## Rule ordering

Rules are matched in the order they are declared in `[site.routes]`, and the first match wins. Place more specific rules before catch-all rules:

```toml
[site.routes]
# Specific rule first
"/blog/archive" = { redirect = "/blog/all", status = 301 }
# Catch-all after
"/blog/*" = { redirect = "https://blog.example.com/{splat}", status = 301 }
```

A request to `/blog/archive` matches the first rule and redirects to `/blog/all`.

## Destination URLs

Redirect targets can be:

| Format        | Example                    | Behavior                     |
| ------------- | -------------------------- | ---------------------------- |
| Absolute URL  | `https://example.com/page` | Redirect to external site    |
| Absolute path | `/new-path`                | Redirect within same site    |
| Relative path | `../other`                 | Resolved against request URL |

Only `http:` and `https:` protocols are allowed. Query strings can be included in the target:

```toml
"/search" = { redirect = "/find?source=redirect" }
```

## Choosing a status code

For static sites on Stencila Sites, use these status codes as a guide:

| Status | Use when | Notes |
| ------ | -------- | ----- |
| `301` | The old URL has moved permanently and clients can update bookmarks and search indexes | Common for page renames and site restructures |
| `302` | You need a temporary redirect with broad legacy compatibility | Some clients may change the request method |
| `303` | You explicitly want the follow-up request to use `GET` | Most useful for form or action endpoints rather than static pages |
| `307` | You need a temporary redirect and want to preserve the original request method | Default for Stencila Sites because it is the safest temporary redirect |
| `308` | The move is permanent and you want to preserve the original request method | Permanent counterpart to `307` |

For most static-site page moves:

- use `301` for permanent page moves
- use `307` for temporary moves or maintenance redirects
- use `302` only when you specifically want classic temporary redirect behavior
- use `303` mainly for non-page workflows
- use `308` when permanent method-preserving behavior matters

# Per-directory redirects

Place a `_redirect.json` file in any directory to redirect requests for that exact route.

```json
{
  "location": "/docs/",
  "status": 301
}
```

| Field      | Type    | Required | Description                                     |
| ---------- | ------- | -------- | ----------------------------------------------- |
| `location` | string  | yes      | Target URL (relative or absolute)                |
| `status`   | integer | no       | HTTP status code: `301`, `302`, `303`, `307`, or `308`. Defaults to `307` |

Per-directory redirects only trigger for route requests (paths ending with `/` that have no `index.html`). They match the exact directory — there is no inheritance to subdirectories.

If a `_redirect.json` file exists for a route that also has a config redirect in `stencila.toml`, the per-directory file takes precedence.

## Example

A `_redirect.json` in the `old-docs/` directory:

```json
{
  "location": "/docs/",
  "status": 301
}
```

| Request            | Result                                         |
| ------------------ | ---------------------------------------------- |
| `/old-docs/`       | 301 → `/docs/`                                 |
| `/old-docs/guide/` | 404 (no `_redirect.json` in `old-docs/guide/`) |

# Resolution order

For static sites hosted on Stencila Sites, redirects are resolved in this order:

1. **Static file** — serve the file if it exists
2. **Per-directory `_redirect.json`** — checked for route requests when no `index.html` exists
3. **Root-level `_redirects.json`** — config redirects checked for any unmatched path
4. **404** — if nothing matches

Static files always take priority — redirects never override an existing file.

# Constraints

| Constraint           | Detail                                                      |
| -------------------- | ----------------------------------------------------------- |
| Splat position       | Must be the last segment (`/a/*` ✓, `/a/*/b` ✗)             |
| Splats per rule      | One `*` maximum                                             |
| Placeholder matching | Single path segment only (no `/` within match)              |
| Status codes         | `301`, `302`, `303`, `307`, `308` (others default to `302`) |
| Protocol safety      | Only `http`/`https` destinations allowed                    |
| File priority        | Redirects never override existing files                     |
| Precedence           | Per-directory `_redirect.json` > config `_redirects.json`   |
