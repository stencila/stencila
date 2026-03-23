# Print and PDF token review reference

Use this file when reviewing paged output, PDF behavior, and any theme artifact that changes page size, page margins, or running headers and footers.

For a comprehensive current list of builtin print tokens, use the CLI:

```sh
stencila themes tokens --scope print
```

Use machine-readable output when you need to inspect or post-process the inventory:

```sh
stencila themes tokens --scope print --as json
```

## Important token groups

Use the CLI listing to find exact names in groups such as:

- `--page-width` and `--page-height`
- `--page-margin-*`
- `--page-top-*-content`
- `--page-bottom-*-content`
- `--page-margin-font-*`
- `--page-border-*`
- `--page1-*`

## Review principles

- Keep exportable page tokens at top-level `:root` when they need to affect non-web targets.
- Treat page tokens nested inside `@media` or `@supports` as a portability risk if the user expects PDF, DOCX, or other non-web outputs to match.
- Review page-fit effects on tables, figures, and code blocks, not only the page tokens themselves.
- PDF and DOCX page mappings are related but not identical, so review both when both matter.

## How print tokens are applied

- PDF is produced from HTML with print media enabled.
- Page tokens control page size, margins, headers, footers, and related print presentation.
- Margin box content can reference injected metadata variables such as `--document-title`, `--document-authors`, `--document-doi`, and `--document-date`.
- Keep exportable page tokens in top-level `:root`; tokens nested inside `@media` or `@supports` are useful for browser print rendering but are not exported as general non-web theme values.

## How to use the CLI and this reference together

1. Use `stencila themes tokens --scope print` to get the current token inventory.
2. Verify exact page and margin token names from the CLI output against what the artifact uses.
3. Use this reference to decide whether token placement and groups are correct.
4. Flag exportable tokens that are incorrectly nested inside `@media` or `@supports`.

## Useful reminders

- PDF is produced from HTML with print media enabled.
- Margin box content can reference metadata variables such as `--document-title`, `--document-authors`, `--document-doi`, and `--document-date`.
- Browser print refinements may still be valid, but they should not be confused with exportable cross-target theme values.
- `@media print` rules affect web print behavior but are not exported to non-web targets.

## Validation workflow

When the theme must support print or PDF, recommend checks such as:

1. render a PDF or print preview
2. verify page size and margins
3. compare running headers and footers across regular pages and the first page
4. confirm table, figure, and code blocks still fit the page width
5. re-check DOCX separately if page-layout parity matters, because PDF and DOCX mappings are not identical
6. run `stencila themes validate theme.css` after editing the file; use `--strict` if unknown tokens should fail
