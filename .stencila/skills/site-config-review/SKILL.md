---
name: site-config-review
description: Review a Stencila site configuration (stencila.toml) for correctness, completeness, best practices, and rendered appearance. Use when asked to review, audit, validate, check, or assess a site config, stencila.toml, site routes, redirects, site layout, layout presets, site nav, navigation, site access, access control, or site deployment readiness.
keywords:
  - review site config
  - audit stencila.toml
  - validate site configuration
  - check site routes
  - review site layout
  - review site navigation
  - review site access
  - site deployment readiness
  - site best practices
  - layout preset review
  - route verification
  - redirect validation
  - access control review
  - site visual review
  - specimen page
  - site snap
allowed-tools: read_file glob grep snap shell ask_user
---

## Overview

Review a Stencila site configuration (`stencila.toml`) for structural correctness, route validity, dependency completeness, layout quality, best practices, and rendered appearance. The primary mode is **assess and report**: identify concrete issues, missing configuration, risky assumptions, and deployment blockers.

This is a **review-only skill**. Do not use it to create a new site configuration from scratch. Use it when the user wants an assessment of an existing `stencila.toml` or a proposed configuration change.

Use these references:

- [`references/site-configuration.md`](references/site-configuration.md) for the complete field reference with types, defaults, and examples for every subsection

- [`references/snap-tool.md`](references/snap-tool.md) for visual verification of layout and component changes

Use `stencila config check` to validate the configuration.

Use `stencila config show` to inspect the resolved configuration after validation and verify that changes took effect as intended.

## Required Inputs

| Input | Required | Description |
|---|---|---|
| Site config file | Required | Path to `stencila.toml` or the config content to review |
| Review scope | Optional | Focus area: full review, routes only, layout only, etc. |
| Deployment target | Optional | Whether the site targets Stencila Cloud, self-hosted, or local preview |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output | Description |
|---|---|
| Review findings | Prioritized list of blocking, important, and optional findings |
| Verdict | Overall assessment: ready to deploy, acceptable with caveats, or not ready |

## Review Dimensions

Evaluate the configuration against these dimensions:

### 1. Structural Validation

- Does the TOML parse without errors?
- Are all fields recognized? (`deny_unknown_fields` is enforced on all config structs)
- Do field values match expected types and patterns?
  - `domain`: lowercase domain pattern `^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$`
  - `workspace.id`: pattern `ws` + 10 lowercase alphanumeric chars
  - `workspace.watch`: pattern `wa` + 10 lowercase alphanumeric chars
- Are enum values valid? (presets: docs/blog/landing/api; access levels: public/subscriber/password/team; spread modes: grid/zip; redirect statuses: 301/302/303/307/308)

### 2. Route Verification

- Do file routes point to files that exist relative to `site.root` (or workspace root if no root)?
- Do redirect targets use internal routes (starting with `/`)?
- Do spread templates have matching `arguments` for all non-reserved placeholders? (Reserved: `{tag}`, `{branch}`, `{i}`)
- Are nav routes (`site.nav`) all internal (starting with `/`)?
- Do nav routes correspond to configured or discoverable file routes?
- Are there duplicate route keys?
- Are route keys properly formatted (starting and ending with `/` for access config)?

### 3. Dependency Checks

Cloud-dependent features require `workspace.id`:
- `reviews` enabled without `workspace.id` → warning
- `uploads` enabled without `workspace.id` → warning
- `remotes` enabled without `workspace.id` → warning
- `access` with non-public levels without `workspace.id` → warning
- `domain` set without `workspace.id` → likely misconfiguration

### 4. Layout Review

- Is the preset appropriate for the site type? (docs: left nav + right TOC; blog: simpler; landing: no sidebars, `main.width = "none"`, `main.padding = "none"`, `main.title = false`; api: left nav + simplified right)
- Do component references resolve to built-in types or named components in `[site.layout.components]`?
- Are `rows` used only in horizontal regions (header, top, bottom, footer) and not in sidebars?
- Do layout overrides have valid glob patterns in `routes`?
- Are override presets appropriate for their target routes?
- Does override order make sense? (first matching override wins)
- Does the merge behavior make sense? (explicit config overrides preset; omitted regions inherit; `false` disables)
- Is `[site.layout.main]` configured appropriately? (width, padding, title)
- Is `[site.layout.responsive]` configured? (breakpoint, toggle-style: fixed-edge/header/hamburger)
- Is the specimen config (`site.specimen.layout`) consistent with the main layout?

### 5. Best Practices

- **Redundant settings**: preset provides defaults — are explicit region configs just duplicating preset values?
- **Missing social links**: `social-links` component in layout but no `site.socials` configured
- **Missing copyright info**: `copyright` component in layout but no `site.author` configured
- **Missing logo**: `logo` component in layout but no `site.logo` configured
- **Missing title**: `title` component in layout but no `site.title` configured
- **Key format issues**: `site.icons`, `site.labels`, `site.descriptions` keys — are they using the right format (full route vs bare segment vs label)?
- **Featured content conflicts**: both `icon` and `image` set (only icon displayed)
- **Featured content keys**: do they match nav group routes/labels?
- **Missing logo variants**: `logo` configured but no dark variant for sites with color-mode toggle
- **Search without layout**: `search = true` but no `site-search` component in layout
- **Reviews/uploads without actions**: features enabled but `actions` not configured
- **Access monotonicity**: child routes must not be less restrictive than parents
- **Unused routes**: routes configured but not reachable from navigation
- **Managed keys**: `workspace.id` and `site.domain` should be set via dedicated commands, not `stencila config set`
- **Glide misconfiguration**: `site.glide` enabled but `prefetch` or `cache` values seem extreme
- **Auto-index with explicit index files**: `site.auto-index` enabled for directories that already have index files (harmless but redundant)
- **Override order**: layout overrides where a broad glob (e.g., `/**`) precedes a narrow one (e.g., `/docs/**`), shadowing the narrow override

### 6. Visual Verification

Use `snap` on the `/_specimen/` route to verify rendered appearance when a Stencila server is running:

- **Layout rendering**: do all configured regions render correctly?
- **Header**: logo, nav-menu, search, color-mode in expected positions
- **Sidebars**: nav-tree, toc-tree visible and functional
- **Footer**: nav-groups, copyright, social-links rendered
- **Navigation**: nav items match config, groups expand/collapse
- **Multi-device**: check mobile, tablet, desktop viewports
- **Color scheme**: check both light and dark modes
- **Measurements/tokens**: extract layout measurements to verify dimensions
- **Palette**: verify color harmony across the site

## Steps

1. Read the `stencila.toml` file.
   - If the user provides a path, read that file.
   - If no path is given, search for `stencila.toml` in the current directory and parent directories.
   - If the config cannot be found, ask the user for the path.

2. Perform structural validation.
   - Check TOML syntax and field recognition.
   - Verify field values against expected types and patterns.
   - Run `stencila config check` to validate the configuration against the schema.
   - Run `stencila config show` to confirm the config loads and to inspect the resolved values.
   - If loading fails, report the parse error as a blocking finding.

3. Verify routes.
   - For each file route in `site.routes`, check that the target file exists relative to `site.root` (or workspace root if no `root` is set).
   - For redirect routes, verify the redirect target is an internal route starting with `/`.
   - For spread routes, verify all non-reserved placeholders (`{tag}`, `{branch}`, `{i}` are reserved) have matching `arguments`.
   - Run `stencila site list` to see the full resolved route table and cross-reference with config.
   - Run `stencila site list --expand` to verify spread route expansion produces the expected routes.
   - Check that nav routes in `site.nav` correspond to actual routes.
   - Check for duplicate route keys.

4. Check dependencies.
   - If cloud features (reviews, uploads, remotes, non-public access) are enabled, verify `workspace.id` is set.
   - If `domain` is set, verify `workspace.id` is present.

5. Review layout configuration.
   - Consult [`references/site-configuration.md`](references/site-configuration.md) for preset defaults, region structure, and component types.
   - Check component references resolve to built-in types or names defined in `[site.layout.components]`.
   - Check override patterns are valid globs and that override order is intentional (first match wins).
   - Identify redundant config that duplicates preset defaults.
   - Check region/component consistency (e.g., `social-links` in layout → `site.socials` configured).
   - Verify `rows` is used only in horizontal regions (header, top, bottom, footer), not in sidebars.

6. Check best practices.
   - Scan for the patterns listed in the Best Practices dimension above.
   - Verify key formats in `site.icons`, `site.labels`, `site.descriptions`, `site.featured`.
   - Check for missing logo dark variants when color-mode toggle is present.

7. Visual verification with `snap` (when available).
   - If a Stencila server is running, snap the `/_specimen/` route:
     - `snap(route: "/_specimen/", measure: "site")` — verify layout regions render correctly
     - `snap(route: "/_specimen/", screenshot: true, full_page: true)` — visual overview
     - `snap(route: "/_specimen/", device: "mobile", measure: "site")` — responsive check
     - `snap(route: "/_specimen/", dark: true, screenshot: true)` — dark mode check
     - `snap(route: "/_specimen/", tokens: true, token_prefix: ["layout", "nav"])` — verify layout token values
     - `snap(route: "/_specimen/", palette: true)` — color harmony
   - Also snap the site root (`"/"`) and key content routes.
   - Prefer rendered directory routes for index-like source files: `index.*`, `main.*`, and `README.*` act as the `index.html` for their containing directory, so `docs/README.md`, `docs/main.md`, and `docs/index.md` all render at `"/docs/"`.
   - If snap is unavailable, mark visual verification as "pending" and recommend specific snap commands.

8. Produce structured review output.
   - Separate findings by severity (blocking / important / optional).
   - Cite specific config keys, line references, and snap evidence for each finding.
   - Provide a final verdict or recommended next step.

## Output Guidelines

Structure the response like this:

1. **Config reviewed**: file path and summary of what is configured
2. **What looks good**: strengths and well-configured areas
3. **Blocking findings**: issues that prevent correct operation
4. **Important findings**: issues that should be fixed before deployment
5. **Optional improvements**: polish, simplification, or future enhancements
6. **Visual verification**: snap results or "pending" with recommended commands
7. **Validation commands**: CLI commands to run for further verification
8. **Verdict**: ready to deploy, acceptable with caveats, or not ready

Each finding should cite the specific config key (e.g., `site.layout.header.end`) and, when available, snap evidence.

## Examples

Input: Review our site configuration for deployment readiness.

Output:

1. **Config reviewed**: `stencila.toml` — docs preset, 12 routes, custom domain, access restrictions
2. **What looks good**:
   - Clean preset-based layout with minimal overrides
   - Blog section uses landing preset override appropriately
   - Social links configured for all layout social-links components
3. **Blocking findings**:
   - `site.access` has non-public routes but `workspace.id` is not set — access restrictions will not be enforced without Stencila Cloud hosting
4. **Important findings**:
   - `site.routes./old-docs/` redirects to `/docs/` but uses status 302; use 301 for permanent redirects
   - `site.nav` includes `/api/reference/` but no matching route exists in `site.routes` or as a discoverable file
   - `site.featured.docs` has both `icon` and `image` — only the icon will display
5. **Optional improvements**:
   - `site.layout.header.end` duplicates the docs preset default (`["site-search"]`) — can be removed
   - Consider adding `site.logo.dark` for sites with color-mode toggle
6. **Visual verification**:
   - `snap(route: "/_specimen/", measure: "site")` confirmed all layout regions render; nav-tree shows 10 items matching config
   - `snap(route: "/_specimen/", device: "mobile", measure: "site")` showed sidebar collapses correctly at mobile breakpoint
   - `snap(route: "/_specimen/", dark: true, screenshot: true)` revealed logo is hard to read on dark background — missing dark variant
7. **Validation commands**:
   - `stencila config check` — validate configuration against the schema
   - `stencila config show` — inspect resolved values, verify domain and workspace linkage
   - `stencila site list` — cross-reference all routes
   - `stencila site list --expand` — verify spread route expansion
8. **Verdict**: not ready for production deployment — fix workspace.id for access restrictions. Remaining issues are important but non-blocking.

---

Input: Just check the routes in our stencila.toml.

Output (scoped review):

1. **Config reviewed**: `stencila.toml` routes section — 8 file routes, 2 redirects, 1 spread
2. **What looks good**:
   - File routes all point to existing files
   - Spread route has correct argument mapping
3. **Blocking findings**: none
4. **Important findings**:
   - Route `/changelog/` points to `CHANGELOG.md` which is outside `site.root = "docs"` — file will not be found relative to the site root
   - Redirect `/old-api/` → `/api/` uses status 307 (temporary). If this is permanent, use 301
5. **Verdict**: one route path issue to fix, otherwise routes are valid

## Edge Cases

- If the user asks for review but no `stencila.toml` exists, say so and ask whether they want help creating one (redirect to site config creation).
- If the config is empty or minimal (just `[site]`), note that a bare config is valid — Stencila uses sensible defaults. Review what the defaults will produce.
- If `site.root` is set, all file path checks must be relative to that root, not the workspace root.
- If `snap` cannot be run (no server, no browser), mark visual verification as "pending" and list the specific snap commands to run later. Do not fabricate snap findings.
- If the user provides a partial config or diff, review only the provided portion but note what cannot be verified without the full config.
- If `site.nav` is not configured, navigation is auto-generated from routes — note this as informational, not as a finding.
- If layout overrides use glob patterns, verify the patterns are syntactically valid.
- If `site.featured` keys do not match any nav group route or label, flag as a potential misconfiguration.
- If the config uses `stencila.local.toml` for local overrides, note that local config is typically gitignored and not deployed.
- If `site.glide` is configured, check that prefetch and cache values are reasonable (not excessively large).
- If `site.auto-index` is enabled with `exclude` patterns, verify the patterns are syntactically valid globs.
- If `site.search.include-types` is set, verify the type names are valid (Heading, Paragraph, Datatable, CodeChunk, Figure, Table are the defaults).
