---
title: Stencila Site Configuration
description: A user-friendly guide to configuring published Stencila sites with stencila.toml
---

Stencila sites are configured in `stencila.toml` at the root of your workspace.
The `[site]` section lets you control how your published site looks, how content
is routed, which interactive features are enabled, and how readers move around it.

This guide is intended as a practical, user-facing companion to the more detailed
configuration reference under [`/docs/config/site/`](/docs/config/site/). If you
are just getting started, begin here. If you need every field and type, use the
reference pages.

## Where configuration lives

Your site configuration goes in the workspace root `stencila.toml` file.

```toml
[site]
title = "My Documentation"
root = "docs"
```

Many settings can be written either:

- as a **simple value** for the common case, or
- as a **table** when you need more control.

For example, search can be enabled with a single boolean:

```toml
[site]
search = true
```

or configured in more detail:

```toml
[site.search]
enabled = true
exclude-routes = ["/api/**"]
```

Stencila validates site configuration strictly. Unknown keys in `[site]` and
its nested tables will cause a configuration parse error, so it is worth
checking field names carefully if a config does not load.

## A gentle starting point

You do not need to configure everything up front. A small site can start with
just a title and a content root:

```toml
[site]
title = "My Project Docs"
root = "docs"
```

From there, you can add:

- **branding**: title, author, logo, icons, social links
- **structure**: navigation, labels, descriptions, featured content
- **publishing rules**: routes, access, exclude patterns, output formats
- **layout**: headers, sidebars, footers, search, and responsive behavior
- **interactivity**: reviews, uploads, remotes, actions, and glide navigation

Here is a typical starting configuration for a documentation site:

```toml
[site]
title = "Acme Docs"
domain = "docs.acme.org"
author = "Acme Inc"
root = "docs"
search = true
reviews = true

[site.logo]
default = "logo.svg"
dark = "logo-dark.svg"
alt = "Acme"

[site.socials]
github = "acme/docs"
discord = "acme-community"

[site.labels]
api = "API"
cli = "CLI"

[site.layout]
preset = "docs"
```

## Top-level fields

These are the main fields you are most likely to use in `[site]`.

| Field | Type | What it does |
| --- | --- | --- |
| `domain` | string | Sets the custom domain for the published site, such as `docs.example.org` |
| `title` | string | Sets the site title used by layout components and fallback metadata |
| `author` | string or table | Identifies the site author or organization |
| `logo` | string or table | Sets a site logo, including responsive and dark-mode variants |
| `root` | string | Limits published content to a specific directory |
| `exclude` | array of strings | Excludes matching files from publishing |
| `search` | boolean or table | Enables and configures client-side search |
| `sitemap` | boolean or table | Generates sitemap files for crawlers and other tooling |
| `reviews` | boolean or table | Enables page comments and suggestions |
| `uploads` | boolean or table | Enables file uploads via GitHub PRs |
| `remotes` | boolean or table | Enables adding Google Docs or Microsoft 365 files via PRs |
| `formats` | array of strings | Adds extra output formats for site content |

## Branding and identity

### `domain`

Use `domain` to set the custom domain for your published site.

```toml
[site]
domain = "docs.example.org"
```

This should be a normal hostname. The canonical source of truth is Stencila Cloud,
so this value is best understood as the workspace's local configuration.

### `title`

Use `title` for the human-readable name of your site.

```toml
[site]
title = "Project Atlas"
```

This is used by the `title` layout component and as fallback metadata.

### `author`

You can set `author` as either a simple string or a richer author object.

```toml
[site]
author = "Acme Inc"
```

or:

```toml
[site.author]
type = "Organization"
name = "Acme Inc"
url = "https://acme.com"
```

This is useful for metadata and for components such as copyright.

### `logo`

For simple cases, `logo` can just be an image path:

```toml
[site]
logo = "logo.svg"
```

If you need responsive or dark-mode variants, use a table instead:

```toml
[site.logo]
default = "logo.svg"
dark = "logo-dark.svg"
mobile = "logo-mobile.svg"
dark-mobile = "logo-mobile-dark.svg"
link = "/"
alt = "Acme"
```

Available fields include `default`, `mobile`, `tablet`, `dark`,
`dark-mobile`, `dark-tablet`, `link`, and `alt`.

You do not need to define every variant. Stencila falls back sensibly when a
more specific logo is missing. For example, a dark mobile logo falls back
through `dark`, then `mobile`, then `default`.

## Navigation metadata

These fields help shape the labels and presentation of your navigation without
requiring you to manually define every navigation item.

### `[site.icons]`

Assigns icons to navigation items.

Lookup order is:

1. full route, such as `"/docs/config/"`
2. route without leading and trailing slashes, such as `"docs/config"`
3. label, such as `"Features"`
4. bare segment, such as `config`

```toml
[site.icons]
"/" = "home"
docs = "book"
"Features" = "sparkles"
```

Icons can be written as `"banana"` for the default Lucide set or
`"lucide:banana"` for an explicit library prefix.

### `[site.labels]`

Overrides the label shown for navigation items.

```toml
[site.labels]
cli = "CLI"
api = "API"
"/docs/db/" = "Database"
```

Use this when automatic title-casing is not what you want.

### `[site.descriptions]`

Adds short descriptions for navigation items, mainly used by components such as
`nav-menu`.

```toml
[site.descriptions]
"docs/getting-started" = "Quick start guide"
docs = "Documentation and guides"
```

### `[site.socials]`

Adds social and external links, usually shown with the `social-links` layout
component.

```toml
[site.socials]
github = "org/repo"
discord = "invite-code"
x = "handle"
```

Supported shortcuts include:

- `bluesky = "handle.bsky.social"`
- `discord = "invite"`
- `facebook = "page"`
- `github = "org"` or `"org/repo"`
- `gitlab = "org"` or `"org/repo"`
- `instagram = "handle"`
- `linkedin = "in/name"` or `"company/name"`
- `mastodon = "https://..."` for a full URL
- `reddit = "r/sub"` or `"u/user"`
- `twitch = "channel"`
- `x = "handle"` or `twitter = "handle"`
- `youtube = "@channel"`

If both `x` and `twitter` are provided, `x` takes precedence.

### `[site.featured.<key>]`

Adds featured or promotional content to a nav menu dropdown.

```toml
[site.featured.docs]
badge = "New"
icon = "rocket"
title = "Quick Start Guide"
description = "Get up and running in minutes"
cta = { label = "Get Started", route = "/docs/getting-started/" }
```

This is keyed by the dropdown's parent group, using the same matching rules as
 icons and descriptions. The `title` field is required. Other supported fields
include `badge`, `icon`, `image`, `description`, and `cta`.

If both `icon` and `image` are provided, the icon takes precedence.

## Navigation structure

### `nav`

If you do not define `nav`, Stencila can derive navigation automatically from
your routes and content tree. Define `nav` when you want explicit ordering,
grouping, or labels.

There are three main item forms:

```toml
# 1. Route shorthand
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

Useful fields on nav items include:

- `id` for a stable identifier
- `label` for display text
- `route` for the internal URL
- `children` for groups
- `icon` and `description` for richer presentation
- `section-title` for grouped dropdown layouts

All routes must be internal and start with `/`.

## Content scope and publishing rules

### `root`

`root` sets the directory that should be treated as the source for the site.

```toml
[site]
root = "docs"
```

This changes how routes are calculated. For example, if `root = "docs"`, then
`docs/guide.md` becomes `/guide/` rather than `/docs/guide/`.

### `exclude`

Use `exclude` for file globs that should not be published.

```toml
[site]
exclude = ["**/*.draft.md", "_drafts/**"]
```

Patterns are relative to `root` if it is set, otherwise the workspace root.

### `formats`

Use `formats` when you want extra generated output formats.

```toml
[site]
formats = ["md"]
```

The exact formats available depend on the surrounding publishing pipeline, so
this is usually an advanced setting.

### `[site.access]`

Controls access levels for routes.

```toml
[site.access]
default = "public"
"/data/" = "password"
"/internal/" = "team"
```

Access levels are ordered:

`public` < `subscriber` < `password` < `team`

Route keys must start and end with `/`, and the longest matching prefix wins.
Access rules are cumulative, so child routes should not be less restrictive
than their parents.

### `[site.routes]`

Lets you map routes explicitly to files, redirects, or spread configurations.

```toml
[site.routes]
"/" = "index.md"
"/about/" = "README.md"

"/old-page/" = { redirect = "/new-page/", status = 301 }

"/{region}/" = { file = "report.smd", arguments = { region = ["north", "south"] } }
```

Common uses include:

- choosing a specific file for `/`
- adding redirects from old URLs to new ones
- generating multiple routes from a single templated source document

Redirect status codes include `301`, `302`, `303`, `307`, and `308`.
Spread modes are `grid` for cartesian products and `zip` for positional pairing.
Advanced spread routes can also use reserved placeholders such as `{tag}`,
`{branch}`, and `{i}`.

## Layout and page structure

### `[site.layout]`

The layout system controls where site-level UI appears around your content.
In most cases, the easiest approach is to start from a preset and customize only
what you need.

If you do not choose a preset explicitly, the layout behaves like the `docs`
preset.

#### Presets

| Preset | Best for |
| --- | --- |
| `docs` | documentation sites with left nav, right table of contents, breadcrumbs, and prev/next links |
| `blog` | article or blog pages with a simpler structure |
| `landing` | homepages or landing pages with no sidebars |
| `api` | API reference pages with navigation on the left and a simplified right side |

In practice, these presets differ mainly in which built-in components are placed
in which regions. For example, `docs` includes left navigation, breadcrumbs,
prev/next links, and a right-side table of contents, while `blog` removes the
left sidebar and bottom navigation. The `landing` preset removes sidebars and
also sets `main.width = "none"`, `main.padding = "none"`, and
`main.title = false`.

```toml
[site.layout]
preset = "docs"
```

#### Regions

You can place components into six layout regions:

| Region | Purpose |
| --- | --- |
| `header` | full-width top bar |
| `left-sidebar` | left-side navigation or tools |
| `top` | area above the main content |
| `bottom` | area below the main content |
| `right-sidebar` | table of contents or secondary tools |
| `footer` | full-width footer |

Each region has `start`, `middle`, and `end` slots. Horizontal regions can also
use `rows` for multi-row layouts. Sidebars use only `start`, `middle`, and
`end`.

A region can be omitted to inherit its current behavior, configured as a table,
or set to `false` to disable it.

#### Built-in components

Common built-in components include:

- `logo`
- `title`
- `breadcrumbs`
- `nav-tree`
- `nav-menu`
- `nav-groups`
- `toc-tree`
- `prev-next`
- `color-mode`
- `copyright`
- `social-links`
- `edit-source`
- `edit-on:gdocs`
- `edit-on:m365`
- `copy-markdown`
- `site-search`
- `site-review`

The generated component reference also documents `edit-on` as a configurable
component type.

#### Main content area

Use `[site.layout.main]` to control content width, padding, and title display.

```toml
[site.layout.main]
width = "none"
padding = "none"
title = false
```

#### Route-specific overrides

Use overrides when different parts of your site need different layouts.

```toml
[[site.layout.overrides]]
routes = ["/blog/**"]
preset = "blog"
left-sidebar = false

[[site.layout.overrides]]
routes = ["/"]
preset = "landing"
```

The first matching override wins, so order matters.

Within an override, explicit settings replace the preset or base layout, while
omitted regions inherit from it. Use `false` to disable a region explicitly.

#### Named components

You can define reusable named components under `[site.layout.components]`.

```toml
[site.layout.components.nav-tree]
collapsible = true
depth = 3

[site.layout.components.main-nav]
type = "nav-tree"
collapsible = true
```

If the component name matches a built-in component type, `type` can be omitted.
When you reference a component by name in a region, it must either be a built-in
component type or a name you defined under `[site.layout.components]`.

#### Responsive behavior

Use `[site.layout.responsive]` to control when sidebars collapse and how their
toggle controls work.

```toml
[site.layout.responsive]
breakpoint = 1024
toggle-style = "fixed-edge"
```

Supported toggle styles are `fixed-edge`, `header`, and `hamburger`.

#### Example layout

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
```

## Navigation behavior and search

### `[site.glide]`

Glide enables faster client-side navigation between internal pages.

```toml
[site.glide]
enabled = true
prefetch = 25
cache = 10
```

When enabled, Stencila can preload likely next pages and use smoother
page transitions where the browser supports them.

### `[site.search]`

Search builds a client-side full-text index during site rendering.

```toml
# Simple form
[site]
search = true

# Detailed form
[site.search]
enabled = true
include-types = ["Heading", "Paragraph", "Datatable"]
exclude-routes = ["/api/**", "/internal/**"]
max-text-length = 500
fuzzy = true
```

When `include-types` is not specified, the generated docs describe the default
set as:

`Heading`, `Paragraph`, `Datatable`, `CodeChunk`, `Figure`, and `Table`

Use search when your site is large enough that readers need quick lookup across
pages.

### `[site.sitemap]`

Sitemap generation writes machine-readable route lists for search engines,
validators, and other tools that need to discover published pages.

```toml
# Simple form
[site]
sitemap = true

# Detailed form
[site.sitemap]
enabled = true
formats = ["xml", "txt"]
visibility = "public-only"
exclude-routes = ["/drafts/**", "/internal/**"]
include-lastmod = true
```

The simple boolean form enables sitemap generation with the default format of
`xml`.

Use the detailed form when you want to control:

- `formats`: generate `xml`, `txt`, or both
- `visibility`: choose `public-only` or `all`
- `exclude-routes`: omit matching routes from sitemap output
- `include-lastmod`: include last-modified timestamps where available

In practice:

- `public-only` includes only public routes and excludes the specimen page
- `all` also includes restricted routes and the specimen page
- `txt` produces a plain text sitemap, while `xml` produces the standard XML
  sitemap format used by most crawlers

Use sitemaps when you want external crawlers or downstream tooling to discover
your published routes more reliably.

## Reader interactions and contribution flows

These features are especially useful for team documentation, data publishing,
and collaborative knowledge bases. Several of them require `workspace.id`
because enforcement happens through Stencila Cloud.

In practice, you will usually want `workspace.id` configured before using
reviews, uploads, remotes, restricted access, or custom domain management.

### `[site.reviews]`

Enables page comments and suggestions.

```toml
# Simple form
[site]
reviews = true

# Detailed form
[site.reviews]
enabled = true
public = true
anon = false
types = ["comment", "suggestion"]
min-selection = 10
max-selection = 5000
shortcuts = false
include = ["docs/**"]
exclude = ["api/**"]
```

You can limit where reviews appear with `include` and `exclude`, and you can
control whether they are available on spread routes with `spread-routes`.

### `[site.uploads]`

Enables file uploads through GitHub pull requests.

```toml
# Simple form
[site]
uploads = true

# Detailed form
[site.uploads]
enabled = true
public = false
anon = false
path = "data"
include = ["data/**"]
exclude = ["api/**"]
extensions = ["csv", "json", "xlsx"]
spread-routes = false
```

This is useful for workflows where contributors need to submit datasets or other
files without writing directly to the repository.

### `[site.remotes]`

Enables adding Google Docs or Microsoft 365 documents via GitHub PRs.

```toml
# Simple form
[site]
remotes = true

# Detailed form
[site.remotes]
enabled = true
path = "content"
default-format = "smd"
allowed-formats = ["smd", "md"]
default-sync-direction = "bi"
public = false
anon = false
user-path = true
require-message = false
include = ["docs/**"]
exclude = ["api/**"]
```

This is useful when teams author in cloud editors but want the site backed by a
repository.

### `[site.actions]`

Controls the floating action button zone used by reviews, uploads, and remotes.

```toml
[site.actions]
position = "bottom-right"
direction = "vertical"
mode = "collapsed"
```

`position` can be `bottom-right`, `bottom-left`, `top-right`, or `top-left`.
`direction` can be `vertical` or `horizontal`. `mode` can be `collapsed` or
`expanded`.

## Automatic index pages

### `[site.auto-index]`

When enabled, Stencila can generate index pages for directories that do not
already have their own content file.

```toml
# Simple form
[site]
auto-index = true

# Detailed form
[site.auto-index]
enabled = true
exclude = ["/api/**", "/internal/**"]
```

This is especially useful for documentation trees where some folders exist only
to organize child pages.

## Specimen page

### `[site.specimen]`

The specimen page is a preview surface for site components and styles. It has
its own layout configuration.

```toml
[site.specimen.layout]
preset = "docs"

[site.specimen.layout.header]
start = "logo"
end = ["color-mode"]
```

The specimen layout uses the same layout system as `[site.layout]`.

## How to choose what to configure

If you are building a typical docs site, a good progression is:

1. set `title`, `root`, and `domain`
2. choose `layout.preset = "docs"`
3. enable `search = true`
4. add `logo`, `socials`, and any custom `labels`
5. define `nav` only if the automatic structure is not enough
6. add `reviews`, `uploads`, or `remotes` only if you need contribution flows

## See also

- [Site config reference](/docs/config/site/)
- [Site layout reference](/docs/config/site/layout/)
- [Site layout components](/docs/config/site/layout/components/)
