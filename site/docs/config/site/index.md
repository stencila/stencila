---
title: Site Config
description: Configuration for a site
---

Configuration for a site

Site settings are associated with a workspace (see `WorkspaceConfig`).
The workspace ID is used to identify the site in Stencila Cloud, e.g.
```toml
# Basic site config with domain, root, excludes, and routes
[site]
domain = "docs.example.org"
root = "docs"
exclude = ["**/*.draft.md", "_drafts/**"]

[site.routes]
"/" = "index.md"
"/about/" = "README.md"
```

# `domain`

**Type:** `string` (optional)

**Pattern:** `^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$`

Custom domain for the site

This is a cached value that is kept in sync with Stencila Cloud
when site details are fetched or the domain is modified.
The canonical source is the Stencila Cloud API.

# `title`

**Type:** `string` (optional)

Site title

Used by the Title component and as fallback metadata.
When not specified, the Title component will render empty, e.g.
```toml
# Set a site title
[site]
title = "My Documentation"
```

# `author`

**Type:** `AuthorSpec` (optional)

Site author

Used as the default copyright holder and for site metadata.
Can be a simple string or a full Author object for richer metadata, e.g.
```toml
# Simple author name
[site]
author = "Acme Inc"

# Full author metadata
[site.author]
type = "Organization"
name = "Acme Inc"
url = "https://acme.com"
```

# `logo`

**Type:** [`LogoConfig`](logo.md) (optional)

Site logo configuration

# `icons`

**Type:** `object` (optional)

Icon assignments for nav items

Applied during nav construction. Icons specified directly on NavItem take precedence.

**Key formats** (lookup order, most to least specific):
1. Full route: `"/docs/config/"` - exact match
2. Without slashes: `"docs/config"` - flexible path matching
3. Label: `"Features"` - matches nav item labels
4. Bare segment: `config` - matches last path segment (e.g., both `/docs/config/` and `/api/config/`)

For unambiguous matching of nested routes, use full routes or paths without leading slash.

**Icon format**: `"banana"` (default lucide set) or `"lucide:banana"` (explicit library), e.g.
```toml
# Map nav routes and labels to icons
[site.icons]
"/" = "home"
"docs/config" = "bolt"  # Specific: only /docs/config/
docs = "book"           # Matches any route ending in /docs/
"Features" = "sparkles" # Matches by label
```

# `descriptions`

**Type:** `object` (optional)

Descriptions for navigation items

Used by nav components (e.g., `nav-menu`) to display descriptions.
Descriptions specified directly on NavItem take precedence.

**Key formats** (lookup order, most to least specific):
1. Full route: `"/docs/config/"` - exact match
2. Without slashes: `"docs/config"` - flexible path matching
3. Label: `"Features"` - matches nav item labels
4. Bare segment: `config` - matches last path segment

For unambiguous matching of nested routes, use full routes or paths without leading slash, e.g.
```toml
# Add short descriptions for nav items
[site.descriptions]
"docs/getting-started" = "Quick start guide"  # Specific
docs = "Documentation and guides"             # Any /docs/ route
"Features" = "Explore all capabilities"       # By label
```

# `socials`

**Type:** `object` (optional)

Social/external links for the site

Keyed by platform name (github, discord, linkedin, etc.). Values can be
shortcuts (expanded automatically) or full URLs. Used by the `social-links`
component. Icons are automatically determined from the platform key.

Supported platforms and shortcuts:

- `bluesky = "handle.bsky.social"` → bsky.app/profile/...
- `discord = "invite"` → discord.gg/invite
- `facebook = "page"` → facebook.com/page
- `github = "org"` or `"org/repo"` → github.com/org or github.com/org/repo
- `gitlab = "org"` or `"org/repo"` → gitlab.com/org or gitlab.com/org/repo
- `instagram = "handle"` → instagram.com/handle
- `linkedin = "in/name"` or `"company/name"` → linkedin.com/...
- `mastodon` → requires full URL (federated)
- `reddit = "r/sub"` or `"u/user"` → reddit.com/...
- `twitch = "channel"` → twitch.tv/channel
- `x = "handle"` or `twitter = "handle"` → x.com/handle
- `youtube = "@channel"` → youtube.com/@channel

Note: `twitter` and `x` are treated as aliases. Both are accepted,
but `x` takes precedence if both are specified.

Order is preserved - links appear in the order defined, e.g.
```toml
# Social links with shortcuts and full URLs
[site.socials]
github = "org/repo"
discord = "invite-code"
linkedin = "company/name"
x = "handle"
mastodon = "https://mastodon.social/@handle"
```

# `featured`

**Type:** `object` (optional)

Featured/promotional content for nav-menu dropdowns

Displays promotional content in the dropdown panel of a nav group.
Keyed by the **dropdown's parent group** (not leaf items).

**Key formats** (lookup order, most to least specific):
1. Full route: `"/docs/config/"` - exact match
2. Without slashes: `"docs/config"` - flexible path matching
3. Label: `"Features"` - matches nav group labels
4. Bare segment: `config` - matches last path segment

For unambiguous matching, use full routes or paths without leading slash, e.g.
```toml
# Featured content keyed by docs dropdown
[site.featured.docs]  # Matches /docs/ dropdown (bare segment)
title = "Quick Start"
description = "Get up and running"
cta = { label = "Start", route = "/docs/getting-started/" }
```

# `nav`

**Type:** `array` (optional)

Site navigation structure

Defines the hierarchical navigation used by nav-tree and prev-next components.
If not specified, navigation is auto-generated from document routes, e.g.
```toml
# Custom nav ordering with groups
[site]
nav = [
  "/",
  { label = "Docs", children = [
    "/docs/getting-started/",
    "/docs/configuration/",
  ]},
  "/about/",
]
```

# `root`

**Type:** `ConfigRelativePath` (optional)

Root directory for site content

Path relative to the config file containing this setting.
When set, only files within this directory will be published
to the site, and routes will be calculated relative to this
directory rather than the workspace root.

Example: If set to "docs" in /myproject/stencila.toml,
then /myproject/docs/guide.md → /guide/ (not /docs/guide/)

# `exclude`

**Type:** `array` (optional)

Glob patterns for files to exclude when publishing

Files matching these patterns will be excluded from publishing.
Exclude patterns take precedence over include patterns.
Patterns are relative to `root` (if set) or the workspace root.
Default exclusions (`.git/`, `node_modules/`, etc.) are applied automatically.

Example: `["**/*.draft.md", "temp/**"]`

# `routes`

**Type:** `object` (optional)

Custom routes for serving content

Routes map URL paths to files, redirects, or spread configurations.
The key is the URL path (or path template for spreads), and the value can be:
- A simple string for the file path: `"/about/" = "README.md"`
- An object for redirects: `"/old/" = { redirect = "/new/", status = 301 }`
- An object for spreads: `"/{region}/" = { file = "report.smd", arguments = { region = ["north", "south"] } }`, e.g.
```toml
# Routes for files, redirects, and spread variants
[site.routes]
"/" = "index.md"
"/about/" = "README.md"
"/old-page/" = { redirect = "/new-page/", status = 301 }
"/{region}/{species}/" = { file = "report.smd", arguments = { region = ["north", "south"], species = ["ABC", "DEF"] } }
```

# `layout`

**Type:** [`LayoutConfig`](layout/) (optional)

Site layout configuration

# `glide`

**Type:** [`GlideConfig`](glide.md) (optional)

Glide configuration for client-side navigation

# `formats`

**Type:** `array` (optional)

Additional formats to generate alongside HTML

Controls which format files are generated during site rendering and
which format-specific buttons are displayed. When a format is not
in this list, its corresponding button (e.g., copy-markdown) is hidden.

Default: `["md"]` (generates page.md files), e.g.
```toml
# Enable or disable additional formats
[site]
formats = ["md"]  # Generate page.md files, show copy-markdown button
formats = []      # No additional formats, hide format buttons
```

# `reviews`

**Type:** [`ReviewsConfig`](reviews.md) (optional)

Site reviews configuration


***

This documentation was generated from [`site.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/site.rs) by [`generate.rs`](https://github.com/stencila/stencila/blob/main/rust/config/src/bin/generate.rs).
