# Snap tool for theme visual verification

The `snap` tool captures screenshots and measurements of pages served by a running Stencila server. It returns a screenshot image for visual inspection and optional JSON measurement data, resolved CSS token values, color palette extraction, and assertion results.

Use `snap` throughout theme creation to verify how changes actually render rather than relying on CSS reading alone. You can snap both the `/_specimen` route and the user's own document or site routes.

## The `/_specimen` route

The `/_specimen` route is a stable, deterministic page that renders representative examples of every major content type — typography, headings, lists, blockquotes, code blocks, math, images, tables, figures, admonitions, and thematic breaks. It is the canonical target for theme visual QA because it exercises all token families in one page.

## User pages

You can also snap the user's actual documents and site pages to verify the theme on real content. Use `route: "/"` for the site root or `route: "/path/to/doc"` for a specific page. Snapping both `/_specimen` and user pages gives the most complete picture.

## Examples

```
# Baseline screenshot of the specimen page with token extraction
# Always use token_prefix with tokens — without it the output includes
# hundreds of tokens and is likely to be truncated
snap(route: "/_specimen", full_page: true, tokens: true, token_prefix: ["text", "heading", "color", "surface"])

# Verify specific token families took effect
snap(route: "/_specimen", tokens: true, token_prefix: ["color", "font", "heading"])

# Check dark mode rendering
snap(route: "/_specimen", full_page: true, dark: true)

# Check light mode explicitly
snap(route: "/_specimen", full_page: true, light: true)

# Responsive checks across devices
snap(route: "/_specimen", full_page: true, devices: ["mobile", "laptop", "desktop"])

# Extract color palette for harmony review
snap(route: "/_specimen", palette: true)

# Focus on a specific element type
snap(route: "/_specimen", selector: "stencila-code-chunk")
snap(route: "/_specimen", selector: "stencila-table")

# Use the theme measurement preset
snap(route: "/_specimen", measure: "theme")

# Assert specific CSS property values
snap(route: "/_specimen", assert: ["css(.title).fontSize>=28px"])

# Snap the user's actual document or site pages
snap(route: "/", full_page: true)
snap(route: "/my-doc", full_page: true, dark: true)
```

## Key parameters for theme work

| Parameter        | Use for                                                                                               |
| ---------------- | ----------------------------------------------------------------------------------------------------- |
| `route`          | Page to snap — use `"/_specimen"` for the canonical QA page or a document path for real content       |
| `screenshot`     | Whether to capture a screenshot (defaults to true)                                                    |
| `full_page`      | Capture the full scrollable page instead of just the viewport                                         |
| `selector`       | CSS selector to capture or measure a specific element                                                 |
| `device`         | Device preset for viewport: `"mobile"`, `"tablet"`, `"laptop"`, `"desktop"`, `"tablet-landscape"`     |
| `devices`        | Multiple device presets for batch measurement                                                         |
| `dark` / `light` | Force a color scheme                                                                                  |
| `measure`        | Measurement preset — use `"theme"` for theme-critical regions                                         |
| `tokens`         | Extract resolved CSS custom property values. **Always combine with `token_prefix`**                   |
| `token_prefix`   | Filter token output to matching prefixes (requires `tokens`). Always provide this to avoid truncation |
| `palette`        | Extract the page's color palette                                                                      |
| `assert`         | Assertion expressions to evaluate, e.g. `["css(.title).fontSize>=28px"]`                              |
| `resize`         | Screenshot resize mode: `"never"`, `"auto"`, `"optimize"` (default)                                   |
