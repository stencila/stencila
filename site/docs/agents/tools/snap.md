---
title: Snap Tool
description: Tool for capturing screenshots and collecting structured measurements of pages served by Stencila, including resolved CSS tokens, layout metrics, assertions, and color palettes.
---

The `snap` tool collects structured measurement data from pages served by a running Stencila server. By default it returns JSON — resolved CSS token values, element measurements, assertion results, color palette, and selector-match diagnostics — without capturing a screenshot. Screenshots are opt-in with `screenshot: true` and return a PNG image alongside the JSON.

Use `snap` to verify how changes actually render rather than relying on CSS reading alone. Start with measurement-only runs to establish a baseline, then add targeted screenshots when visual confirmation is needed.

## `snap`

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `route` | string | | Page to snap — a route on the running Stencila server or a document path on disk. Defaults to `"/"` |
| `screenshot` | boolean | | Whether to capture a screenshot image. Defaults to **false** (structured data only) |
| `full_page` | boolean | | Capture the full scrollable page instead of just the viewport. Defaults to false |
| `selector` | string | | CSS selector to capture or measure a specific element. Takes precedence over full-page capture. Missing selectors return a diagnostic instead of erroring |
| `device` | string | | Device preset for viewport dimensions: `"mobile"`, `"tablet"`, `"laptop"`, `"desktop"`, `"tablet-landscape"` |
| `width` | integer | | Custom viewport width in pixels. Height falls back to default if omitted |
| `height` | integer | | Custom viewport height in pixels. Width falls back to default if omitted |
| `dark` | boolean | | Force dark color scheme. Mutually exclusive with `light` and `print`. Defaults to false |
| `light` | boolean | | Force light color scheme. Mutually exclusive with `dark` and `print`. Defaults to false |
| `print` | boolean | | Emulate print media with A4 dimensions. Incompatible with `dark` and `light`. Defaults to false |
| `measure` | string | | Measurement preset determining which selectors to measure. See [measurement presets](#measurement-presets) |
| `tokens` | boolean | | Extract resolved CSS custom property (theme token) values, grouped by token family. **Requires at least one `token_prefix`** or the tool returns a validation error. Defaults to false |
| `token_prefix` | array of strings | | CSS custom property prefixes to filter token output (required when `tokens` is true). Use narrow prefixes like `["text", "heading", "surface"]` — broad prefixes like `"color"` match 100+ primitive palette tokens |
| `palette` | boolean | | Extract the page's color palette. Defaults to false |
| `assert` | array of strings | | Assertion expressions to evaluate. Supports `>=`, `<=`, `==`, `!=`, `>`, `<` for numeric values and `~=` (contains) for strings like fontFamily. Example: `["css(stencila-paragraph).fontSize>=16px"]` |
| `wait_for` | string | | Wait for a CSS selector to exist before capturing |
| `delay` | integer | | Additional delay in milliseconds after page is ready |
| `resize` | string | | Screenshot resize mode: `"never"`, `"auto"` (default — resizes only above 8000px hard limit), `"optimize"` (downscales aggressively to reduce payload size) |
| `max_image_dimension` | integer (min: 1) | | Maximum screenshot dimension in pixels after resize. Used with `"auto"` (default 8000px) or `"optimize"` (default 4096px) |

## The `/_specimen` route

The `/_specimen` route is a stable, deterministic page that renders representative examples of every major content type — typography, headings, lists, blockquotes, code blocks, math, images, tables, figures, admonitions, and thematic breaks. It is the canonical target for theme visual QA because it exercises all token families in one page.

## User pages

You can also snap the user's actual documents and site pages to verify the theme on real content. Use `route: "/"` for the site root or `route: "/path/to/doc"` for a specific page. Snapping both `/_specimen` and user pages gives the most complete picture.

## Measurement presets

| Preset     | What it measures                                                                 |
| ---------- | -------------------------------------------------------------------------------- |
| `auto`     | Automatically selects relevant regions based on the page structure               |
| `document` | Document content area — paragraphs, headings, lists, code, tables, figures       |
| `site`     | Site chrome — header, navigation, footer, sidebar                                |
| `all`      | Everything: combines document and site presets                                   |
| `header`   | Header region only                                                               |
| `nav`      | Navigation region only                                                           |
| `main`     | Main content region only                                                         |
| `footer`   | Footer region only                                                               |
| `theme`    | Theme-critical layout regions with summaries and diagnostics (most useful for QA)|

Unknown `measure` values produce a validation error listing the valid options.

The `theme` preset output is ordered with summaries and diagnostics first, followed by verbose per-element CSS. The summaries are typically the most useful part — scan those before diving into raw values.

## Output fields

Every snap call returns JSON with some or all of these fields depending on what was requested:

- **`selector_matched`** — whether the target selector was found on the page (present even without a screenshot)
- **measurements** — per-element computed styles when a `measure` preset is active
- **tokens** — resolved CSS custom property values grouped by family (when `tokens: true`)
- **palette** — extracted color palette (when `palette: true`)
- **assertions** — pass/fail results for each assertion expression
- **diagnostics** — informational messages (e.g. unmatched selectors on the route — these are route facts, not theme bugs)

## Examples

```
# Measurement-only baseline — no screenshot (default behavior)
# Extract specific token families to verify they took effect
snap(route: "/_specimen", tokens: true, token_prefix: ["text", "heading", "surface"])

# Verify specific token families resolved correctly
snap(route: "/_specimen", tokens: true, token_prefix: ["font", "heading", "line-height"])

# Holistic layout review with the theme measurement preset
# Returns summaries and diagnostics first, then verbose CSS details
snap(route: "/_specimen", measure: "theme")

# Site chrome review (header, nav, footer)
snap(route: "/_specimen", measure: "site")

# Extract color palette for harmony and contrast review
snap(route: "/_specimen", palette: true)

# Assert specific CSS property values — works best for numeric properties
snap(route: "/_specimen", assert: ["css(stencila-paragraph).fontSize>=16px", "css(stencila-heading).lineHeight>=1.2"])

# Assert string CSS properties with the ~= (contains) operator
snap(route: "/_specimen", assert: ["css(stencila-heading).fontFamily~=Source Serif"])

# Full-page screenshot for initial visual overview (opt-in)
snap(route: "/_specimen", screenshot: true, full_page: true)

# Check dark mode rendering with a screenshot
snap(route: "/_specimen", screenshot: true, full_page: true, dark: true)

# Check light mode explicitly
snap(route: "/_specimen", screenshot: true, full_page: true, light: true)

# Responsive checks — use separate calls per device
snap(route: "/_specimen", measure: "theme", device: "mobile")
snap(route: "/_specimen", measure: "theme", device: "desktop")

# Focus on a specific element type
snap(route: "/_specimen", screenshot: true, selector: "stencila-code-block")
snap(route: "/_specimen", screenshot: true, selector: "stencila-table")

# Snap the user's actual document or site pages
snap(route: "/", measure: "theme")
snap(route: "/my-doc", screenshot: true, full_page: true, dark: true)

# Print mode emulation (A4 dimensions) — incompatible with dark/light
snap(route: "/_specimen", screenshot: true, print: true)

# Width-only viewport (height falls back to default)
snap(route: "/_specimen", measure: "theme", width: 480)
```

## Practical tips

- **Measurement-first workflow**: start with measurement-only runs (the default) to establish baselines, then add screenshots only when visual confirmation is needed.
- **Token verification is the strongest feature**: always use narrow `token_prefix` values like `["text", "heading", "surface"]`. Broad prefixes like `"color"` match 100+ primitive palette tokens and flood context.
- **Use `measure: "theme"` for holistic layout review** and `measure: "site"` for site chrome (header, nav, footer).
- **Diagnostics about unmatched selectors are informational** — they describe what the route contains, not bugs in the theme. The `selector_matched` field in the output tells you whether an element was found, even without a screenshot.
- **Missing selectors in screenshots are handled gracefully** — the tool returns a diagnostic instead of hard-erroring.
- **Contrast calculations** work on elements with explicit backgrounds. Transparent descendants report "background color is unavailable" — this is expected, not an error.
- **Screenshots work best for**: initial full-page overview, print-mode reflow checks, and specific element captures with unique selectors.
- **Avoid full-page screenshots of very tall pages** for typography review — use selector-targeted snaps instead (e.g. `selector: "stencila-heading"`).
- **Prefer unique selectors** for element screenshots — when multiple elements match, only the first is captured but measurements cover all matches.
- **Assertions are best for numeric CSS properties** (fontSize, lineHeight, width, height, marginTop, etc.). Use the `~=` operator for string matching (e.g. fontFamily).
- **Theme changes are cached for ~30 seconds** by the server. Batch CSS edits before snapping to see them all reflected.
- **Responsive testing**: use separate `device: "mobile"` and `device: "desktop"` calls — there is no batch multi-device parameter.
- **Unknown `measure` values** produce a validation error listing the valid options — use only the documented presets.
