# Stencila theme review reference

Use this file for the architecture, terminology, and cross-target constraints that matter when reviewing an existing or proposed Stencila theme artifact.

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

- document typography modules such as links, headings, paragraphs, and lists
- content modules such as code, tables, figures, images, quotes, math, and breaks
- Stencila component modules such as admonitions, citations, plots, references, and works metadata
- site modules such as layout, nav-menu, nav-tree, nav-groups, breadcrumbs, toc-tree, prev-next, logo, site title, copyright, site search, and review widgets
- print support via page tokens and print-specific rules

## Review constraints that affect correctness

- If no theme is specified, Stencila resolves workspace `theme.css` while walking up from the document path, then user `default.css`, then builtin `stencila.css`.
- If a named theme is requested, user themes are searched before builtin themes.
- Only top-level `:root` custom properties are exported to non-web targets. Tokens inside `@media` or `@supports` may affect web rendering but are not exported to DOCX, PDF, email, or plot targets.
- `var()`, `calc()`, and `color-mix()` expressions are resolved before theme values are mapped into non-web outputs.
- Runtime theme resolution is cached for roughly 30 seconds, so stale previews can look like theme bugs.

## Review priorities

Use these questions to guide the review:

1. Is the artifact targeting the right surface: document, site, plot, print/PDF, or a mix?
2. Does it start from semantic tokens where possible?
3. Are exact module-specific token names verified instead of guessed?
4. Are values that must transfer across targets kept at top-level `:root`?
5. Does the artifact overuse selectors where tokens would be clearer or more portable?
6. Are target-specific limits for PDF, DOCX, email, Python, and R described accurately?

## Cross-target reminders for reviewers

- Document themes often need separate confirmation for HTML, PDF, DOCX, and email.
- Site chrome styling does not imply PDF parity.
- Only `--plot-*` tokens transfer into Python and R plot theming.
- PDF and DOCX page behavior are related but not identical.
- Avoid approving claims of full parity unless the relevant targets have been checked.

## Base theme loading

Stencila automatically loads `base.css` before any theme CSS. Review user themes expecting a file that starts directly with optional external font imports and `:root` overrides.

Do not treat the absence of `@import url("./base.css")` as a problem. Treat the presence of that import in a workspace theme as a review finding.
