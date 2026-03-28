# Snap tool for theme visual verification

The `snap` tool collects structured measurement data from pages served by a running Stencila server. By default it returns JSON — resolved CSS token values, element measurements, assertion results, color palette, and selector-match diagnostics — without capturing a screenshot. Screenshots are opt-in with `screenshot: true` and return a PNG image alongside the JSON.

Use `snap` throughout theme creation to verify how changes actually render rather than relying on CSS reading alone. Start with measurement-only runs to establish a baseline, then add targeted screenshots when visual confirmation is needed.

## The `/_specimen` route

The `/_specimen` route is a stable, deterministic page that renders representative examples of every major content type — typography, headings, lists, blockquotes, code blocks, math, images, tables, figures, admonitions, and thematic breaks. It is the canonical target for theme visual QA because it exercises all token families in one page.

## User pages

You can also snap the user's actual documents and site pages to verify the theme on real content. Use `route: "/"` for the site root or `route: "/path/to/doc"` for a specific page. Snapping both `/_specimen` and user pages gives the most complete picture.

## Examples

```
# Measurement-only baseline — no screenshot (default behavior)
# Extract specific token families to verify they took effect
snap(route: "/_specimen", tokens: true, token_prefix: ["text", "heading", "surface"])

# Holistic layout review with the theme measurement preset
# Returns summaries and diagnostics first, then verbose CSS details
snap(route: "/_specimen", measure: "theme")

# Site chrome review (header, nav, footer)
snap(route: "/_specimen", measure: "site")

# Full-page screenshot for initial visual overview (opt-in)
snap(route: "/_specimen", screenshot: true, full_page: true)

# Check dark mode rendering with a screenshot
snap(route: "/_specimen", screenshot: true, full_page: true, dark: true)

# Check light mode explicitly
snap(route: "/_specimen", screenshot: true, full_page: true, light: true)

# Responsive checks — use separate calls per device
snap(route: "/_specimen", measure: "theme", device: "mobile")
snap(route: "/_specimen", measure: "theme", device: "desktop")

# Extract color palette for harmony review
snap(route: "/_specimen", palette: true)

# Focus on a specific element type
snap(route: "/_specimen", screenshot: true, selector: "stencila-code-block")
snap(route: "/_specimen", screenshot: true, selector: "stencila-table")

# Assert specific CSS property values (numeric)
snap(route: "/_specimen", assert: ["css(stencila-paragraph).fontSize>=16px"])

# Assert string CSS properties with the ~= (contains) operator
snap(route: "/_specimen", assert: ["css(stencila-heading).fontFamily~=Source Serif"])

# Snap the user's actual document or site pages
snap(route: "/", measure: "theme")
snap(route: "/my-doc", screenshot: true, full_page: true, dark: true)

# Print mode emulation (A4 dimensions) — incompatible with dark/light
snap(route: "/_specimen", screenshot: true, print: true)

# Width-only viewport (height falls back to default)
snap(route: "/_specimen", measure: "theme", width: 480)
```

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

## Key parameters for theme work

| Parameter        | Use for                                                                                               |
| ---------------- | ----------------------------------------------------------------------------------------------------- |
| `route`          | Page to snap — use `"/_specimen"` for the canonical QA page or a document path for real content       |
| `screenshot`     | Whether to capture a screenshot (defaults to **false** — structured data only)                        |
| `full_page`      | Capture the full scrollable page instead of just the viewport                                         |
| `selector`       | CSS selector to capture or measure a specific element. Missing selectors return a diagnostic instead of erroring |
| `device`         | Device preset for viewport: `"mobile"`, `"tablet"`, `"laptop"`, `"desktop"`, `"tablet-landscape"`     |
| `width`          | Custom viewport width in pixels (height falls back to default if omitted)                             |
| `height`         | Custom viewport height in pixels (width falls back to default if omitted)                             |
| `dark` / `light` | Force a color scheme (mutually exclusive; incompatible with `print`)                                  |
| `print`          | Emulate print media with A4 dimensions                                                                |
| `measure`        | Measurement preset — see the table above                                                              |
| `tokens`         | Extract resolved CSS custom property values. **Requires at least one `token_prefix`** or the tool returns a validation error |
| `token_prefix`   | CSS custom property prefixes to filter token output (requires `tokens`). Use narrow prefixes like `["text", "heading", "surface"]` — broad prefixes like `"color"` match 100+ primitive palette tokens |
| `palette`        | Extract the page's color palette                                                                      |
| `assert`         | Assertion expressions, e.g. `["css(stencila-paragraph).fontSize>=16px"]`. Supports `>=`, `<=`, `==`, `!=`, `>`, `<` for numeric values and `~=` (contains) for strings like fontFamily |
| `resize`         | Screenshot resize mode: `"never"`, `"auto"` (default — resizes only above 8000px hard limit), `"optimize"` (downscales aggressively) |
| `max_image_dimension` | Maximum screenshot dimension in pixels after resize. Used with `"auto"` (default 8000px) or `"optimize"` (default 4096px) |
| `wait_for`       | Wait for a CSS selector to exist before capturing                                                     |
| `delay`          | Additional delay in milliseconds after page is ready                                                  |

## Output fields

Every snap call returns JSON with some or all of these fields depending on what was requested:

- **`selector_matched`** — whether the target selector was found on the page (present even without a screenshot)
- **measurements** — per-element computed styles when a `measure` preset is active
- **tokens** — resolved CSS custom property values grouped by family (when `tokens: true`)
- **palette** — extracted color palette (when `palette: true`)
- **assertions** — pass/fail results for each assertion expression
- **diagnostics** — informational messages (e.g. unmatched selectors on the route — these are route facts, not theme bugs)

## Practical tips

- **Measurement-first workflow**: start with measurement-only runs (the default) to establish baselines, then add screenshots only when visual confirmation is needed.
- **Token verification is the strongest feature**: always use narrow `token_prefix` values like `["text", "heading", "surface"]`. Broad prefixes like `"color"` match 100+ primitive palette tokens and flood context.
- **Use `measure: "theme"` for holistic layout review** and `measure: "site"` for site chrome (header, nav, footer).
- **Diagnostics about unmatched selectors are informational** — they describe what the route contains, not bugs in the theme.
- **Contrast calculations** work on elements with explicit backgrounds. Transparent descendants report "background color is unavailable" — this is expected, not an error.
- **Screenshots work best for**: initial full-page overview, print-mode reflow checks, and specific element captures with unique selectors.
- **Avoid full-page screenshots of very tall pages** for typography review — use selector-targeted snaps instead (e.g. `selector: "stencila-heading"`).
- **Assertions are best for numeric CSS properties** (fontSize, lineHeight, width, height, marginTop, etc.). Use the `~=` operator for string matching (e.g. fontFamily).
- **Theme changes are cached for ~30 seconds** by the server. Batch CSS edits before snapping to see them all reflected.
- **Responsive testing**: use separate `device: "mobile"` and `device: "desktop"` calls — there is no batch multi-device parameter.
