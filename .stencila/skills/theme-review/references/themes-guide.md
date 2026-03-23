# Stencila theme review reference

Use this file for the architecture, terminology, cross-target constraints, and resolution rules that matter when critically reviewing an existing or proposed Stencila theme artifact.

## What to review in a Stencila theme

A Stencila theme combines:

- design tokens expressed as CSS custom properties
- optional CSS rules for document, node, plot, print, and site styling
- translation hooks that let the same theme influence non-web outputs such as DOCX, email, PDF, and plots

Review from the token layer outward. Semantic tokens are the stable public API and should usually be the first place to look. Node-specific, plot-specific, print-specific, and site-specific module tokens are appropriate only when the semantic layer is not enough. Broad selector-based CSS should be treated as a deliberate fallback, not the default.

## Theme architecture to keep in mind

The built-in theme system is layered:

- **Primitive tokens** provide raw colors, spacing values, font stacks, radii, shadows, and other low-level ingredients.
- **Semantic tokens** provide the stable theme API for document-wide choices such as text, headings, code, surfaces, spacing, layout, and borders.
- **Module-specific tokens** provide targeted overrides for document nodes, plots, print/page behavior, and site components.

Major module groups include:

- token layers and root styles (`tokens-primitive.css`, `tokens-semantic.css`, `root.css`, `browsers.css`, `pages.css`)
- document typography modules such as links, headings, paragraphs, and lists
- content modules such as code, tables, datatables, figures, images, quotes, math, and breaks
- Stencila component modules such as admonitions, citations, diagrams, plots, references, and works metadata
- site modules such as layout, nav-menu, nav-tree, nav-groups, breadcrumbs, toc-tree, prev-next, logo, site title, copyright, site search, social-links, edit controls, and review widgets
- print support via `pages.css` and module-level `@media print`

## Review constraints that affect correctness

- If no theme is specified, Stencila resolves workspace `theme.css` while walking up from the document path, then user `default.css`, then builtin `stencila.css`.
- If a named theme is requested, user themes are searched before builtin themes.
- If `--plot-theme` is set to a named preset other than `custom` or `none`, preset CSS is merged before variables are extracted.
- Only top-level `:root` custom properties are exported to non-web targets. Tokens inside `@media` or `@supports` may affect web rendering but are not exported to DOCX, PDF, email, or plot targets.
- `var()`, `calc()`, and `color-mix()` expressions are resolved before theme values are mapped into non-web outputs.
- Colors are normalized and lengths are converted into target-specific units.
- Computed variables may be merged with document metadata overrides such as `--document-*`.
- Runtime theme resolution is cached for roughly 30 seconds, so stale previews can look like theme bugs.

## Review priorities

Use these questions to guide the review:

1. Is the artifact targeting the right surface: document, site, plot, print/PDF, or a mix?
2. Does it start from semantic tokens where possible?
3. Are exact module-specific token names verified instead of guessed?
4. Are values that must transfer across targets kept at top-level `:root`?
5. Does the artifact overuse selectors where tokens would be clearer or more portable?
6. Are target-specific limits for PDF, DOCX, email, Python, and R described accurately?
7. Does the theme handle dark-mode variants appropriately for web outputs?
8. Does the artifact avoid including `@import url("./base.css")`?

## Cross-target reminders for reviewers

- Document themes often need separate confirmation for HTML, PDF, DOCX, and email.
- Site chrome styling does not imply PDF parity.
- Only `--plot-*` tokens transfer into Python and R plot theming.
- PDF and DOCX page behavior are related but not identical.
- Avoid approving claims of full parity unless the relevant targets have been checked.
- Dark mode applies only to web and HTML-derived outputs; DOCX and email do not support dark mode.

## Dark-mode review guidance

Many tokens have `*-dark` variants applied automatically via `prefers-color-scheme: dark`. When reviewing:

- Check whether light-mode color values would look wrong on a dark background and whether the `*-dark` variant is set.
- Plot, surface, and text tokens are the most common dark-variant gaps.
- Use `stencila themes tokens --scope semantic` or `--scope plot` to see which tokens have dark variants.
- Dark variants are web-only; do not require them for DOCX or email targets.

## Base theme loading

Stencila automatically loads `base.css` before any theme CSS. Review user themes expecting a file that starts directly with optional external font imports and `:root` overrides.

- Do not treat the absence of `@import url("./base.css")` as a problem.
- Treat the presence of that import in a workspace theme as a review finding — it is unnecessary and may cause issues.
- The builtin themes (`stencila.css`, `tufte.css`, `latex.css`) follow this same pattern — none import `base.css`.

## Document vs site theme review

### Document themes

Expect document themes to start with semantic tokens plus selected node/module tokens such as headings, paragraphs, lists, links, code, tables, figures, citations, references, plots, and print/page tokens.

### Site themes

Expect site themes to include the document foundation plus site-specific token families such as `layout`, `nav-menu`, `nav-tree`, `nav-groups`, `breadcrumbs`, `toc-tree`, `prev-next`, `logo`, `title`, `site-search`, `copyright`, `social-links`, `edit-on`, `edit-source`, `copy-markdown`, and `site-review` where applicable.

Use `site-token-families.md` for exact names and naming quirks. If the exact token name is uncertain or the family is not localized, recommend verifying it with `stencila themes tokens` rather than approving it.

## Target-specific review guidance

### Web and HTML plots

- Stencila reads `--plot-*` variables from `:root` and builds a plot token bundle for JavaScript renderers.
- Theme changes trigger plot recompilation in the browser.
- If `--plot-theme: none` is set, renderers keep their defaults.

### PDF and print

- PDF is produced from HTML with print media enabled.
- `pages.css` and module-level `@media print` rules control page size, margins, headers, footers, and print-specific styling.
- Margin box content can reference injected metadata variables such as `--document-title`, `--document-authors`, and `--document-date`.

### Email

- Theme tokens are converted into MJML attributes and email-safe HTML/CSS.
- Safe areas include typography, links, code, quotes, tables, work metadata, spacing, surfaces, simple borders, and radii.
- Flag claims of complex browser-only layout or interaction support in email output.

### Word (DOCX)

- Theme tokens map into Word paragraph, character, table, and page layout styles.
- Mapped modules include semantic tokens plus headings, paragraphs, code, links, quotes, lists, tables, figures, works metadata, and pages.
- Known gaps include admonitions, breaks, citations, labels, and references; datatables, diagrams, images, plots, and math do not map as DOCX theme modules.
- Flag assumptions of full parity with web output.

### Python and R plots

- Only `--plot-*` tokens are sent to Python and R kernels.
- General document tokens do not theme Python/R plots unless the corresponding `--plot-*` tokens are also defined.
- Some plot tokens are not mapped equally across targets. For example, `--plot-subtitle-size` is not currently mapped in matplotlib, and several interaction-oriented plot tokens are not applied in R.
- Flag approximate cross-renderer parity claims that have not been validated.
