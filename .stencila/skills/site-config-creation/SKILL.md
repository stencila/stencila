---
name: site-config-creation
description: Create or update the [site] section of stencila.toml for published Stencila sites. Use when asked to configure site domain, title, author, logo, icons, labels, descriptions, socials, featured content, navigation, routes, access, layout presets and regions, glide, search, formats, reviews, uploads, remotes, actions, auto-index, or specimen. Covers reading existing TOML, generating valid configuration, editing while preserving comments and formatting, and snap-based visual verification.
keywords:
  - stencila.toml
  - site configuration
  - domain
  - title
  - logo
  - navigation
  - routes
  - redirects
  - access
  - layout preset
  - search
  - reviews
  - uploads
  - remotes
allowed-tools: read_file write_file edit_file apply_patch glob grep shell snap ask_user
---

## Overview

Help the user create or update the `[site]` section of `stencila.toml` for a published Stencila site. This skill covers all site configuration subsections — from basic metadata (domain, title, author) through navigation, layout, search, access control, and interactive features (reviews, uploads, remotes).

Use these references for field details:

- [`references/site-configuration.md`](references/site-configuration.md) for the complete field reference with types, defaults, and examples for every subsection

- [`references/snap-tool.md`](references/snap-tool.md) for visual verification of layout and component changes

Use `stencila config check` to validate the configuration.

Use `stencila config show` to inspect the resolved configuration after validation and verify that changes took effect as intended.

## Core rules

- Always inspect the existing `stencila.toml` before making changes.
- Preserve existing comments, formatting, and unrelated sections when editing.
- Use `edit_file` for targeted TOML edits rather than rewriting the entire file.
- Generate valid TOML — quote keys containing special characters, use correct array and table syntax.
- Validate configuration after changes with `stencila config check`.
- Use `snap` for visual verification of layout, navigation, and component changes when a server is running.
- Do not invent asset files (logos, images); ask the user for paths or use clearly marked placeholders.
- When a field supports both simple and detailed forms (search, reviews, uploads, remotes, auto-index), use the simple form when defaults suffice.

## Steps

1. **Inspect the current state.**
   - Read the existing `stencila.toml` (typically at the workspace root).
   - Note which `[site]` fields already exist and their current values.
   - Identify what the user wants to add or change.

2. **Determine the scope of changes.**
   - Identify which subsections are affected (top-level fields, nav, layout, routes, etc.).
   - If the request is ambiguous (e.g., "set up my site"), ask the user what they need before generating configuration.
   - Use [`references/site-configuration.md`](references/site-configuration.md) to look up correct field names, types, and value formats.

3. **Generate or update the TOML configuration.**
   - For new subsections, add them in a logical order within the file.
   - For existing subsections, edit only the affected fields.
   - Use the correct TOML syntax for each field type:
     - Simple strings: `title = "My Site"`
     - Arrays: `exclude = ["**/*.draft.md", "temp/**"]`
     - Tables: `[site.layout]` with fields below
     - Inline tables: `cta = { label = "Start", route = "/docs/" }`
     - Array of tables: `[[site.layout.overrides]]`

4. **Validate the configuration.**
   - Run `stencila config check` to validate the configuration and catch schema or value errors.
   - Then run `stencila config show` to inspect the resolved configuration and verify the final effective values.
   - Check for validation errors (invalid domain, unknown component names, invalid route patterns).

5. **Visually verify layout and component changes (when applicable).**
   - If the change affects layout, navigation, or visual components and a Stencila server is running, use `snap` to verify.
   - See [`references/snap-tool.md`](references/snap-tool.md) for the typical verification workflow.
   - If `snap` is unavailable, mark visual verification as pending and recommend specific commands.

## Required Inputs

| Input | Required | Description |
|---|---|---|
| User's site configuration goals | Required | What the user wants to configure (domain, layout, nav, etc.) |
| Existing `stencila.toml` content | Required | Current file content (read from workspace) |
| Asset paths (logos, images) | Optional | Paths to logo files, social images, etc. |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output | Description |
|---|---|
| Updated `stencila.toml` | The modified configuration file |
| Validation results | Output from `stencila config check` confirming the configuration is valid, plus `stencila config show` output showing the resolved values |
| Visual verification | Snap results showing layout/component changes render correctly (when available) |

## Subsection guidance

Use this section for quick patterns and common examples. For complete field details, defaults, and additional forms, check [`references/site-configuration.md`](references/site-configuration.md).

### Domain, title, author, logo

These are top-level `[site]` fields. Set them directly:

```toml
[site]
domain = "docs.example.org"
title = "My Documentation"
author = "Acme Inc"
logo = "logo.svg"
```

For responsive logos, use the table form:

```toml
[site.logo]
default = "logo.svg"
dark = "logo-dark.svg"
mobile = "logo-mobile.svg"
alt = "Acme Documentation"
```

### Navigation

If not specified, navigation is auto-generated from document routes. For custom ordering:

```toml
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

Use `[site.icons]`, `[site.labels]`, and `[site.descriptions]` to customize nav appearance without restructuring.

### Layout

Start with a preset, then override specific regions:

```toml
[site.layout]
preset = "docs"

[site.layout.header]
end = ["site-search", "color-mode"]

[[site.layout.overrides]]
routes = ["/"]
preset = "landing"
```

Available presets: `docs`, `blog`, `landing`, `api`. Built-in components: `logo`, `title`, `breadcrumbs`, `nav-tree`, `nav-menu`, `nav-groups`, `toc-tree`, `prev-next`, `color-mode`, `copyright`, `edit-source`, `edit-on:gdocs`, `edit-on:m365`, `copy-markdown`, `site-search`, `site-review`, `social-links`.

### Routes

Use `[site.routes]` to map routes to files, add redirects, or generate route variants from arguments.

```toml
[site.routes]
"/" = "index.md"
"/old-docs/" = { redirect = "/docs/", status = 301 }
"/{region}/report/" = { file = "reports/regional.smd", arguments = { region = ["americas", "emea"] } }
```

### Access control

Use `[site.access]` to control who can view specific routes, with a default access level and per-route overrides.

```toml
[site.access]
default = "public"
"/internal/" = "team"
"/data/" = "password"
```

Route keys must start and end with `/`.

### Interactive features (reviews, uploads, remotes)

These require `workspace.id` to be set. Use simple boolean form when defaults suffice:

```toml
[site]
reviews = true
uploads = true
remotes = true
```

Use detailed form for fine-grained control. Position is configured via `[site.actions]`.

## Examples

Input: Set up a basic site configuration for a documentation project.

Output:

```toml
[site]
title = "Project Documentation"
author = "Project Team"
search = true

[site.layout]
preset = "docs"
```

Input: Add GitHub and Discord social links to our site.

Output: Edit the existing `stencila.toml` to add:

```toml
[site.socials]
github = "org/repo"
discord = "invite-code"
```

Input: Add search to an existing docs site while preserving comments and unrelated settings.

Before:

```toml
[workspace]
id = "acme-docs"

[site]
title = "Acme Docs"

[site.layout]
preset = "docs"
```

After:

```toml
[workspace]
id = "acme-docs"

[site]
title = "Acme Docs"
search = true

[site.layout]
preset = "docs"
```

Input: Configure a landing page for the root with docs layout for everything under /docs/.

Output:

```toml
[site.layout]
preset = "landing"

[[site.layout.overrides]]
routes = ["/docs/**"]
preset = "docs"
```

Input: Add a redirect from /old-path/ to /new-path/ with a 301 status.

Output: Add to `[site.routes]`:

```toml
"/old-path/" = { redirect = "/new-path/", status = 301 }
```

Input: Set up access control so internal docs require team membership but everything else is public.

Output:

```toml
[site.access]
default = "public"
"/internal/" = "team"
```

Input: Enable reviews with custom settings — public but require GitHub auth, only on docs pages.

Output:

```toml
[site.reviews]
enabled = true
public = true
anon = false
include = ["docs/**"]
```

Input: Configure the header to show logo, nav menu, search, and color mode toggle.

Output:

```toml
[site.layout.header]
start = "logo"
middle = "nav-menu"
end = ["site-search", "color-mode"]
```

## Edge Cases

- If `stencila.toml` does not exist, create it with the necessary sections. Include `[workspace]` if needed for features requiring `workspace.id`.
- If the user asks for reviews, uploads, or remotes but `workspace.id` is not set, warn that these features require it and ask whether to add a placeholder.
- If the user provides a domain that does not match the validation regex, suggest a corrected format.
- If layout component names are misspelled or unknown, suggest the correct built-in component name from the known list.
- If the user wants both a landing page root and docs layout for subpages, use layout overrides — do not try to set two presets at the top level.
- If nav routes do not start with `/`, warn that only internal routes are supported in site navigation.
- If access route keys do not end with `/`, add the trailing slash and explain the requirement.
- If `site.root` is set, remind the user that routes and file paths are relative to that directory.
- If `snap` is unavailable, mark visual verification as pending, rely on `stencila config check` for validation and `stencila config show` for inspection, and recommend specific snap commands.
- If the user wants to remove a subsection entirely, delete the relevant TOML lines rather than setting empty values.
- When editing inline tables or arrays of tables, be careful with TOML syntax — inline tables must be on one line, array-of-tables uses `[[double brackets]]`.
