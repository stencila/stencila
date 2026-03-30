---
title: Snap Tool for Site Config Verification
description: Using the snap tool to visually verify site layout, navigation, and component changes after modifying stencila.toml
---

The `snap` tool collects structured measurement data and optional screenshots from pages served by a running Stencila server. Use it to verify that site configuration changes (layout, navigation, components) render as expected.

## Key parameters for site config work

| Parameter | Type | Description |
|---|---|---|
| `route` | string | Page to snap. Defaults to `"/"`. Use `"/_specimen"` for the canonical QA page |
| `screenshot` | boolean | Capture a screenshot (opt-in, default false) |
| `full_page` | boolean | Full scrollable page instead of viewport |
| `selector` | string | CSS selector to capture a specific element |
| `device` | string | Viewport preset: `"mobile"`, `"tablet"`, `"laptop"`, `"desktop"`, `"tablet-landscape"` |
| `dark` / `light` | boolean | Force color scheme |
| `measure` | string | Measurement preset: `"auto"`, `"document"`, `"site"`, `"all"`, `"header"` |
| `tokens` | boolean | Extract resolved CSS token values (requires `token_prefix`) |
| `token_prefix` | array of strings | CSS custom property prefixes to filter tokens |
| `palette` | boolean | Extract the page's color palette |
| `assert` | array of strings | Assertion expressions to evaluate |

## Measurement presets relevant to site config

| Preset | What it measures |
|---|---|
| `site` | Site chrome — header, navigation, footer, sidebar |
| `header` | Header region only |
| `all` | Everything: document + site |
| `auto` | Automatically selects relevant regions |

## Typical snap workflow for site config changes

1. **Before changes**: Snap the current state to establish a baseline.
   ```
   snap(route: "/", measure: "site", screenshot: true)
   ```

2. **After layout changes**: Verify header, sidebar, and footer regions.
   ```
   snap(route: "/docs/some-page/", measure: "site", screenshot: true)
   ```

3. **After nav changes**: Check navigation rendering.
   ```
   snap(route: "/", selector: "stencila-nav-menu", screenshot: true)
   snap(route: "/docs/", selector: "stencila-nav-tree", screenshot: true)
   ```

4. **Responsive check**: Verify mobile layout.
   ```
   snap(route: "/", device: "mobile", measure: "site", screenshot: true)
   ```

5. **Dark mode check**: Verify dark mode rendering.
   ```
   snap(route: "/", dark: true, measure: "site", screenshot: true)
   ```

6. **Specimen page**: Exercise all components on the QA page.
   ```
   snap(route: "/_specimen", measure: "all", screenshot: true)
   ```

## When snap is unavailable

If `snap` cannot be run (no running Stencila server, route not available), mark visual verification as **pending** and rely on `stencila config show` for validation. Recommend specific snap commands to run once the server is available. Do not fabricate visual findings.
