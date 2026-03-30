---
title: Stencila Site Configuration Reference
description: Complete reference for all [site] subsections in stencila.toml
---

## File location

The configuration file is `stencila.toml` at the workspace root. The `[site]` section controls all published site behavior.

## Top-level fields

| Field | Type | Description |
|---|---|---|
| `domain` | string | Custom domain (e.g., `docs.example.org`). Must match `^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$` |
| `title` | string | Site title. Used by the Title layout component and as fallback metadata |
| `author` | string or table | Site author. Simple string (`"Acme Inc"`) or full Author object with `type`, `name`, `url` |
| `logo` | string or table | Site logo. Simple path (`"logo.svg"`) or responsive config with `default`, `mobile`, `tablet`, `dark`, `dark-mobile`, `dark-tablet`, `link`, `alt` |
| `root` | string | Root directory for site content, relative to workspace root |
| `exclude` | array of strings | Glob patterns for files to exclude from publishing |
| `search` | boolean or table | Enable client-side full-text search |
| `formats` | array of strings | Additional formats to generate (`["md"]`) |

## `[site.icons]`

Maps nav routes/labels to icon names. Key formats (lookup order):
1. Full route: `"/docs/config/"`
2. Without slashes: `"docs/config"`
3. Label: `"Features"`
4. Bare segment: `config`

Icon format: `"banana"` (default lucide set) or `"lucide:banana"` (explicit library).

```toml
[site.icons]
"/" = "home"
docs = "book"
"Features" = "sparkles"
```

## `[site.labels]`

Custom labels for nav items, overriding auto-generated labels from route segments. Same key format lookup order as icons.

```toml
[site.labels]
cli = "CLI"
api = "API"
"/docs/db/" = "Database"
```

## `[site.descriptions]`

Descriptions for nav items, used by nav-menu dropdowns. Same key format lookup order as icons.

```toml
[site.descriptions]
"docs/getting-started" = "Quick start guide"
docs = "Documentation and guides"
```

## `[site.socials]`

Social/external links. Keys are platform names, values are shortcuts or full URLs. Order is preserved.

Supported platforms and shortcuts:
- `bluesky = "handle.bsky.social"` → bsky.app/profile/...
- `discord = "invite"` → discord.gg/invite
- `facebook = "page"` → facebook.com/page
- `github = "org"` or `"org/repo"` → github.com/...
- `gitlab = "org"` or `"org/repo"` → gitlab.com/...
- `instagram = "handle"` → instagram.com/handle
- `linkedin = "in/name"` or `"company/name"` → linkedin.com/...
- `mastodon` → requires full URL
- `reddit = "r/sub"` or `"u/user"` → reddit.com/...
- `twitch = "channel"` → twitch.tv/channel
- `x = "handle"` or `twitter = "handle"` → x.com/handle (`x` takes precedence)
- `youtube = "@channel"` → youtube.com/@channel

```toml
[site.socials]
github = "org/repo"
discord = "invite-code"
x = "handle"
```

## `[site.featured.<key>]`

Featured/promotional content for nav-menu dropdowns. Keyed by dropdown's parent group (same lookup as icons).

```toml
[site.featured.docs]
badge = "New"
icon = "rocket"
title = "Quick Start Guide"
description = "Get up and running in minutes"
cta = { label = "Get Started", route = "/docs/getting-started/" }
```

Fields: `badge` (string), `icon` (string), `image` (string, path relative to site root), `title` (string, required), `description` (string), `cta` (table with `label` and `route`).

## `site.nav`

Navigation structure as an array. Three item forms:

```toml
# 1. Route string shorthand
nav = ["/", "/docs/", "/about/"]

# 2. Link with explicit label
nav = [
  { label = "Home", route = "/" },
  { label = "Docs", route = "/docs/" },
]

# 3. Group with children
nav = [
  "/",
  { label = "Docs", children = [
    "/docs/getting-started/",
    "/docs/configuration/",
  ]},
  { label = "Guides", route = "/guides/", children = [
    "/guides/deployment/",
  ]},
]
```

NavItem fields: `id` (optional stable identifier), `label`, `route`, `children` (for groups), `icon`, `description` (for links), `section-title` (for groups in nav-menu dropdowns).

All routes must be internal (start with `/`).

## `[site.access]`

Route access restrictions. Access levels: `public` < `subscriber` < `password` < `team`.

```toml
[site.access]
default = "public"
"/data/" = "password"
"/internal/" = "team"
```

Route keys must start and end with `/`. Longest prefix match wins.

## `[site.routes]`

Custom routes mapping URL paths to files, redirects, or spread configurations.

```toml
[site.routes]
# File route
"/" = "index.md"
"/about/" = "README.md"

# Redirect
"/old-page/" = { redirect = "/new-page/", status = 301 }

# Spread (multi-variant)
"/{region}/" = { file = "report.smd", arguments = { region = ["north", "south"] } }
"/{year}/{quarter}/" = { file = "quarterly.smd", spread = "zip", arguments = { year = ["2024", "2024"], quarter = ["Q1", "Q2"] } }
```

Redirect status codes: 301, 302, 303, 307 (default), 308. Spread modes: `grid` (cartesian product, default), `zip` (positional pairing).

## `[site.layout]`

Controls site page structure. Uses a region-based system with presets.

### Presets

| Preset | Description |
|---|---|
| `docs` | Documentation: nav-tree left, toc-tree right, breadcrumbs, prev-next |
| `blog` | Blog/article: no left sidebar, toc-tree right, no prev-next |
| `landing` | Landing page: no sidebars, centered content |
| `api` | API reference: nav-tree left (flat), no right sidebar |

### Regions

| Region | Position | Sub-regions flow |
|---|---|---|
| `header` | Top, full width | left→right: start, middle, end |
| `left-sidebar` | Left side | top→bottom: start, middle, end |
| `top` | Above main content | left→right: start, middle, end |
| `bottom` | Below main content | left→right: start, middle, end (supports `rows`) |
| `right-sidebar` | Right side | top→bottom: start, middle, end |
| `footer` | Bottom, full width | left→right: start, middle, end |

### Built-in components

`logo`, `title`, `breadcrumbs`, `nav-tree`, `nav-menu`, `nav-groups`, `toc-tree`, `prev-next`, `color-mode`, `copyright`, `edit-source`, `edit-on:gdocs`, `edit-on:m365`, `copy-markdown`, `site-search`, `site-review`, `social-links`

### Main content area

```toml
[site.layout.main]
width = "none"    # "none", "narrow", "default", "wide"
padding = "none"  # "none", "default"
title = false     # show/hide page title
```

### Route-specific overrides

```toml
[[site.layout.overrides]]
routes = ["/blog/**"]
preset = "blog"
left-sidebar = false

[[site.layout.overrides]]
routes = ["/"]
preset = "landing"
```

First matching override wins (order matters).

### Named components

```toml
[site.layout.components.nav-tree]
collapsible = true
depth = 3

[site.layout.components.main-nav]
type = "nav-tree"
collapsible = true
```

### Responsive configuration

```toml
[site.layout.responsive]
breakpoint = 1024
toggle-style = "fixed-edge"
```

### Example

```toml
[site.layout]
preset = "docs"

[site.layout.header]
start = "logo"
middle = "nav-menu"
end = ["site-search", "color-mode"]

[site.layout.left-sidebar]
start = { type = "nav-tree", collapsible = true, depth = 3 }

[site.layout.bottom]
middle = "prev-next"

[site.layout.footer]
start = "nav-groups"
middle = "copyright"
end = "social-links"

[[site.layout.overrides]]
routes = ["/blog/**"]
preset = "blog"
left-sidebar = false
```

## `[site.glide]`

Client-side navigation (AJAX page transitions with View Transitions API).

```toml
[site.glide]
enabled = true    # default: true
prefetch = 25     # max prefetches per session (default: 20)
cache = 10        # max cached pages (default: 10)
```

## `[site.search]`

Client-side full-text search index configuration.

```toml
# Simple form
search = true

# Detailed form
[site.search]
enabled = true
include-types = ["Heading", "Paragraph", "Datatable"]
exclude-routes = ["/api/**", "/internal/**"]
max-text-length = 500
fuzzy = true    # default: true, adds ~1KB per entry
```

Default include-types: `Article`, `Heading`, `Paragraph`, `Datatable`, `CodeChunk`, `Figure`, `Table`.

## `[site.reviews]`

Reader comments and suggestions on site pages. Requires `workspace.id`.

```toml
# Simple form
reviews = true

# Detailed form
[site.reviews]
enabled = true
public = true           # non-team can submit (default: true)
anon = false            # require GitHub auth (default: false)
types = ["comment", "suggestion"]
min-selection = 10      # min chars to trigger widget (default: 1)
max-selection = 5000    # max chars in selection (default: 5000)
shortcuts = false       # keyboard shortcuts
include = ["docs/**"]
exclude = ["api/**"]
```

## `[site.uploads]`

File upload via GitHub PRs. Requires `workspace.id`.

```toml
# Simple form
uploads = true

# Detailed form
[site.uploads]
enabled = true
public = false
anon = false
path = "data"                        # target directory
include = ["data/**"]
exclude = ["api/**"]
extensions = ["csv", "json", "xlsx"] # allowed file types
spread-routes = false                # show on spread routes
```

## `[site.remotes]`

Add Google Docs/M365 documents via GitHub PRs. Requires `workspace.id`.

```toml
# Simple form
remotes = true

# Detailed form
[site.remotes]
enabled = true
path = "content"
default-format = "smd"              # smd, md, html
allowed-formats = ["smd", "md"]
default-sync-direction = "bi"       # from-remote, bi, to-remote
public = false
anon = false
user-path = true
require-message = false
include = ["docs/**"]
exclude = ["api/**"]
```

## `[site.actions]`

Floating action button zone (reviews, uploads, remotes).

```toml
[site.actions]
position = "bottom-right"  # bottom-right, bottom-left, top-right, top-left
direction = "vertical"     # vertical, horizontal
mode = "collapsed"         # collapsed, expanded
```

## `[site.auto-index]`

Auto-generate index pages for directories without content files.

```toml
# Simple form
auto-index = true

# Detailed form
[site.auto-index]
enabled = true
exclude = ["/api/**", "/internal/**"]
```

## `[site.specimen]`

Specimen page configuration for previewing site components and styles.

```toml
[site.specimen.layout]
preset = "docs"

[site.specimen.layout.header]
start = "logo"
end = ["color-mode"]
```

The specimen page layout uses the same `LayoutConfig` system as the main site layout.
