# Snap tool for theme visual verification

The `snap` tool gathers structured measurement data from pages served by a running Stencila server. By default it returns JSON measurements, resolved CSS token values, color palette extraction, and assertion results — **without** taking a screenshot. Screenshots are opt-in with `screenshot: true`.

This measurement-first design means the primary review workflow is: measure, analyze the structured data, form findings, and only then capture targeted screenshots as visual evidence when needed.

Use `snap` during review to gather quantitative evidence for findings rather than relying solely on static CSS analysis. You can snap both the `/_specimen` route and the user's own document or site routes.

## The `/_specimen` route

The `/_specimen` route is a stable, deterministic page that renders representative examples of every major content type — typography, headings, lists, blockquotes, code blocks, math, images, tables, figures, admonitions, and thematic breaks. It is the canonical target for theme visual QA because it exercises all token families in one page.

## User pages

You can also snap the user's actual documents and site pages to verify the theme on real content. Use `route: "/"` for the site root or `route: "/path/to/doc"` for a specific page. Snapping both `/_specimen` and user pages gives the most complete picture during review.

## Measurement-first workflow

Start every review with structured data collection, not screenshots:

1. **Tokens** — verify resolved CSS custom property values with narrow prefixes
2. **Measurements** — use `measure: "theme"` for holistic layout, `measure: "site"` for site chrome
3. **Assertions** — check specific CSS properties against expected values
4. **Palette** — extract colors for harmony and contrast review
5. **Screenshots** — capture only when visual evidence is needed (initial overview, print reflow, element-specific captures)

## Examples

```
# Baseline measurement of the specimen page with token extraction
# tokens: true REQUIRES at least one token_prefix — the tool will error without it
# Use narrow prefixes; broad ones like "color" match 100+ primitive palette tokens
snap(route: "/_specimen", tokens: true, token_prefix: ["text", "heading", "surface"])

# Verify specific token families resolved correctly
snap(route: "/_specimen", tokens: true, token_prefix: ["font", "heading", "line-height"])

# Holistic layout review with theme measurement preset
# Output has summaries and diagnostics first, then verbose CSS — focus on the summaries
snap(route: "/_specimen", measure: "theme")

# Site chrome review (header, nav, footer)
snap(route: "/_specimen", measure: "site")

# Extract color palette for harmony and contrast review
snap(route: "/_specimen", palette: true)

# Assert specific CSS property values — works best for numeric properties
snap(route: "/_specimen", assert: ["css(stencila-paragraph).fontSize>=16px", "css(stencila-heading).lineHeight>=1.2"])

# String matching with the ~= (contains) operator — useful for font families
snap(route: "/_specimen", assert: ["css(stencila-heading).fontFamily~=Source Serif"])

# Check dark mode rendering (opt-in screenshot for visual evidence)
snap(route: "/_specimen", screenshot: true, full_page: true, dark: true)

# Check light mode explicitly
snap(route: "/_specimen", screenshot: true, full_page: true, light: true)

# Responsive checks — use separate calls per device (no batch devices parameter)
snap(route: "/_specimen", measure: "theme", device: "mobile")
snap(route: "/_specimen", measure: "theme", device: "desktop")

# Focus on a specific element type flagged in review
snap(route: "/_specimen", screenshot: true, selector: "stencila-code-block")
snap(route: "/_specimen", screenshot: true, selector: "stencila-table")

# Full-page screenshot for initial overview (opt-in)
snap(route: "/_specimen", screenshot: true, full_page: true)

# Print mode reflow check
snap(route: "/_specimen", screenshot: true, print: true)

# Snap the user's actual document or site pages
snap(route: "/", measure: "theme")
snap(route: "/my-doc", screenshot: true, full_page: true, dark: true)

# Width-only viewport override (height falls back to default)
snap(route: "/_specimen", width: 768, measure: "theme")
```

## Practical tips

- **Token verification is the strongest feature.** Always use narrow prefixes like `["text", "heading", "surface"]`. Broad prefixes like `"color"` match 100+ primitive palette tokens and produce overwhelming output.
- **`measure: "theme"` output** has summaries and diagnostics first, followed by verbose CSS. Focus on the summaries — they are the most useful part.
- **Diagnostics about unmatched selectors** are informational facts about the route, not theme bugs. The `selector_matched` field in the output tells you whether an element was found, even without a screenshot.
- **Missing selectors in screenshots are handled gracefully** — the tool returns a diagnostic instead of hard-erroring.
- **Contrast calculations** work on elements with explicit backgrounds. Transparent descendants report "background color is unavailable" — this is expected, not a theme defect.
- **Avoid full-page screenshots of very tall pages** for typography review — use selector-targeted snaps instead.
- **The 30-second theme cache** means you should batch CSS changes before snapping. After editing theme.css, wait or re-snap to see updated results.
- **Unknown `measure` values** now error instead of silently doing nothing — use only the documented presets.

## Key parameters for theme review

| Parameter        | Use for                                                                                               |
| ---------------- | ----------------------------------------------------------------------------------------------------- |
| `route`          | Page to snap — use `"/_specimen"` for the canonical QA page or a document path for real content       |
| `screenshot`     | Capture a screenshot image (defaults to **false**; set `true` for visual evidence)                    |
| `full_page`      | Capture the full scrollable page instead of just the viewport                                         |
| `selector`       | CSS selector to capture or measure a specific element; returns diagnostic if not found                |
| `device`         | Device preset for viewport: `"mobile"`, `"tablet"`, `"laptop"`, `"desktop"`, `"tablet-landscape"`     |
| `width` / `height` | Custom viewport dimensions in pixels; either can be set alone (the other falls back to default)     |
| `dark` / `light` | Force a color scheme                                                                                  |
| `print`          | Emulate print media (A4 dimensions); incompatible with `dark` and `light`                             |
| `measure`        | Measurement preset: `"auto"`, `"document"`, `"site"`, `"all"`, `"header"`, `"nav"`, `"main"`, `"footer"`, `"theme"` |
| `tokens`         | Extract resolved CSS custom property values. **Requires at least one `token_prefix`** (errors without it) |
| `token_prefix`   | Filter token output to matching prefixes (required when `tokens` is true). Use narrow prefixes        |
| `palette`        | Extract the page's color palette                                                                      |
| `assert`         | Assertion expressions, e.g. `["css(stencila-paragraph).fontSize>=16px"]`; use `~=` for string contains |
| `resize`         | Screenshot resize mode: `"never"`, `"auto"` (default — resizes only above 8000px), `"optimize"`       |
| `selector_matched` | Output field indicating whether the target selector was found — available even without screenshots  |
