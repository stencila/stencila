# Stencila theme creation reference

This reference is self-contained and summarizes the architecture, terminology, constraints, and workflow guidance needed to create or update a Stencila theme. Use it for the overall model, then use `stencila themes tokens` for a comprehensive current token inventory and inspect the focused reference files in this skill directory for application guidance and caveats that the CLI listing does not provide.

## What a Stencila theme is

A Stencila theme combines:

- design tokens expressed as CSS custom properties
- optional CSS rules for document, node, plot, print, and site styling
- translation hooks that let the same theme influence non-web outputs such as DOCX, email, PDF, and plots

Themes should prefer semantic tokens as the public API. Primitive tokens exist, but semantic tokens are the stable layer to override first. Use node-specific, plot-specific, print-specific, or site-specific module tokens only when semantic tokens are not enough.

## Token families and architecture

The built-in theme architecture is layered:

- **Primitive tokens** provide raw colors, spacing values, font stacks, radii, shadows, and similar low-level building blocks.
- **Semantic tokens** provide the stable, public theme API for document-wide choices such as text, heading, code, surfaces, spacing, layout, and borders.
- **Module-specific tokens** provide targeted overrides for specific document nodes, plots, print/page behavior, or site components.

Major module groups include:

- token layers and root styles (`tokens-primitive.css`, `tokens-semantic.css`, `root.css`, `browsers.css`, `pages.css`)
- typography modules such as links, headings, paragraphs, and lists
- content modules such as code, tables, datatables, figures, images, quotes, math, and breaks
- Stencila component modules such as admonitions, citations, diagrams, plots, references, and works metadata
- site modules such as layout, nav-menu, nav-tree, nav-groups, breadcrumbs, toc-tree, prev-next, logo, site title, copyright, site search, social-links, edit controls, and review widgets
- print support via `pages.css` and module-level `@media print`

## Resolution and computation constraints

These implementation details affect how themes behave:

- If no theme is specified, resolution order is: workspace `theme.css` walking up from the document path, then user `default.css`, then base theme (no overrides).
- If a named theme is requested, Stencila searches user themes first, then builtin themes.
- If `--plot-theme` is set to a named preset other than `custom` or `none`, preset CSS is merged before variables are extracted.
- Only top-level `:root` custom properties are exported to non-web targets. Tokens inside `@media` or `@supports` are useful for web rendering but are not exported to DOCX, PDF, email, or plot targets.
- `var()`, `calc()`, and `color-mix()` expressions are resolved before theme values are mapped into non-web outputs.
- Colors are normalized and lengths are converted into target-specific units.
- Computed variables may be merged with document metadata overrides such as `--document-*`.
- Runtime theme resolution is cached for roughly 30 seconds; watch-based tooling is needed for immediate reload behavior.

## Recommended authoring workflow

1. Decide whether the user needs a **plan only** response or an **implement now** response.
2. Decide whether the work is a **document theme**, a **site theme**, a **plot theme**, a **print/PDF theme**, or a theme that must cover several outputs.
3. If updating an existing theme, inspect the current file first and preserve its import style, token naming, selector structure, and unrelated rules.
4. Start from semantic tokens and use `stencila themes tokens --scope semantic` when you need the current builtin inventory.
5. Inspect `stencila themes tokens` with the narrowest useful `--scope` and `--family` filters before emitting exact module-specific token names.
6. Inspect localized token references for workflow and application guidance:
   - `semantic-and-font-tokens.md`
   - `node-token-families.md`
   - `print-and-pdf-tokens.md`
   - `plot-tokens.md`
   - `site-token-families.md`
7. Add node-specific document tokens, plot tokens, print/page tokens, or site token families only when semantic tokens are insufficient.
8. Add CSS rules only when token overrides are insufficient.
9. Keep exported cross-target values in top-level `:root`.
10. Add `@media print` or responsive rules only for web or print-specific presentation, knowing those overrides will not be exported to non-web targets.
11. If assets such as fonts, logos, or icons are needed, reference existing workspace assets when available and avoid inventing missing files.
12. Validate concrete theme files with `stencila themes validate <FILE>` and use `--strict` when unknown tokens should fail.
13. Test the theme in each required output target instead of assuming web behavior will map perfectly everywhere.

## Common semantic token starting points

Good first overrides include:

- `--text-font-family`
- `--heading-font-family`
- `--code-font-family`
- `--text-color-primary`
- `--color-accent`
- `--surface-background`
- `--content-spacing`
- `--content-width`
- `--border-radius-default`
- `--border-color-default`

## Document vs site themes

### Document themes

Document themes usually start with semantic tokens plus selected node/module tokens such as:

- headings
- paragraphs
- lists
- links
- code
- tables
- figures
- citations
- references
- plots
- print/page tokens

### Site themes

Site themes usually need the document foundation plus site-specific token families such as `layout`, `nav-menu`, `nav-tree`, `nav-groups`, `breadcrumbs`, `toc-tree`, `prev-next`, `logo`, `title`, `site-search`, `copyright`, `social-links`, `edit-on`, `edit-source`, `copy-markdown`, and `site-review` where applicable.

Use `site-token-families.md` for exact names that have been localized into this skill. If the exact token name is uncertain or the family is not localized there, describe the family and intended effect instead of guessing a variable.

## Localized reference map

Use the localized reference files in this skill directory as the execution-time documentation set:

- `references/semantic-and-font-tokens.md`
- `references/node-token-families.md`
- `references/print-and-pdf-tokens.md`
- `references/plot-tokens.md`
- `references/site-token-families.md`

These files are the intended operating references for this skill. If an exact token name is missing or may have drifted, verify it with `stencila themes tokens` instead of looking elsewhere.

## Theme CLI commands

Use the CLI as the live source of truth for token inventories and validation:

```sh
# List all builtin tokens
stencila themes tokens

# Filter by scope
stencila themes tokens --scope semantic
stencila themes tokens --scope site
stencila themes tokens --scope plot
stencila themes tokens --scope print

# Filter by family
stencila themes tokens --scope site --family nav-menu

# Machine-readable output for scripts and agents
stencila themes tokens --scope plot --as json

# Validate a theme file
stencila themes validate theme.css

# Treat unknown tokens as errors
stencila themes validate theme.css --strict
```

Use the CLI output to answer “what tokens exist?” Use the localized references to answer “how should these token families be used?” and “what caveats or cross-target limits matter?”

## Fonts and assets

- Start by assigning fonts through semantic tokens such as `--text-font-family`, `--heading-font-family`, `--code-font-family`, and `--math-font-family`.
- If adding custom fonts, define `@font-face` rules or font imports before overriding the relevant font-family tokens.
- Extend existing fallback stacks where practical rather than replacing them with a single font name.
- Do not invent missing asset files. If a font, logo, or icon file is required but absent, either ask for it or use a clearly marked placeholder path for the user to replace.

## Base theme loading

Stencila automatically loads `base.css` before any theme CSS. The base theme is an embedded asset, not a file in the user's workspace:

- In HTML output, it is injected as a separate `<link>` tag before the theme stylesheet.
- For variable computation, base variables are parsed and merged before theme-specific variables, so all base tokens are always available.

Because of this, **user theme files must not include `@import url("./base.css")`**. A workspace `theme.css` or named user theme should start directly with `:root` overrides:

```css
:root {
  --text-font-family: "Your Font", serif;
}
```

If external resources such as web fonts are needed, use `@import url(...)` for the font provider before the `:root` block:

```css
@import url('https://fonts.googleapis.com/css2?family=Inter&display=swap');

:root {
  --heading-font-family: "Inter", sans-serif;
}
```

The builtin themes (`tufte.css`, `latex.css`) follow this same pattern — none of them import `base.css`.

## Target-specific guidance

### Web and HTML plots

- Stencila reads `--plot-*` variables from `:root` and builds a plot token bundle for JavaScript renderers.
- Theme changes trigger plot recompilation in the browser.
- If `--plot-theme: none` is set, renderers keep their defaults.

### PDF and print

- PDF is produced from HTML with print media enabled.
- `pages.css` and module-level `@media print` rules control page size, margins, headers, footers, and print-specific styling.
- Use `print-and-pdf-tokens.md` for exact page token names.
- Margin box content can reference injected metadata variables such as `--document-title`, `--document-authors`, and `--document-date`.

### Email

- Theme tokens are converted into MJML attributes and email-safe HTML/CSS.
- The email encoder focuses on typography and component styling that work reliably in email clients.
- Safe areas include typography, links, code, quotes, tables, work metadata, spacing, surfaces, simple borders, and radii.
- Avoid promising support for complex browser-only layout or interaction patterns in email output.

### Word (DOCX)

- Theme tokens map into Word paragraph, character, table, and page layout styles.
- Computed CSS values are resolved before mapping.
- Mapped modules include semantic tokens plus headings, paragraphs, code, links, quotes, lists, tables, figures, works metadata, and pages.
- Some modules are not yet mapped or are not applicable in DOCX. Known gaps include admonitions, breaks, citations, labels, and references, while datatables, diagrams, images, plots, and math do not map as DOCX theme modules.
- Avoid assuming full parity with web output.

### Python and R plots

- Only `--plot-*` tokens are sent to Python and R kernels.
- General document tokens such as text or heading tokens do not theme Python/R plots unless the corresponding `--plot-*` tokens are also defined.
- Lengths are converted before mapping to renderer-specific units.
- If `--plot-theme: none` is set, kernels skip plot theming.
- Some plot tokens are not mapped equally across targets. For example, `--plot-subtitle-size` is not currently mapped in matplotlib, and several interaction-oriented plot tokens are not applied in R.
- Use `plot-tokens.md` for exact token names before emitting concrete plot variables.
- If exact cross-renderer parity matters, describe the theme as approximate and recommend validation in each target renderer.

## Important limitations to communicate

- Prefer semantic tokens first.
- Keep cross-target tokens at top-level `:root`.
- Do not rely on tokens inside `@media` or `@supports` for non-web outputs.
- Default to simple semantic-token overrides in a `:root` block. Do not include `@import url("./base.css")` since Stencila loads the base theme implicitly.
- Non-web targets support only subsets of the full web theme surface.
- Interactive or web-only modules may not map to DOCX or email.
- Plot theming outside the browser depends on `--plot-*` tokens, not general document tokens alone.
- `--plot-theme: none` disables plot theming for web, Python, and R plot targets.
- Do not promise that arbitrary CSS or every module will translate unchanged to PDF, DOCX, email, Python, or R outputs.
