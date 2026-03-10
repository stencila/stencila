# Print and PDF token review reference

Use this file when reviewing paged output, PDF behavior, and any theme artifact that changes page size, page margins, or running headers and footers.

## Important token groups

Use `stencila themes tokens --scope print` to inspect exact names in groups such as:

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

## Useful reminders

- PDF is produced from HTML with print media enabled.
- Margin box content can reference metadata variables such as `--document-title`, `--document-authors`, `--document-doi`, and `--document-date`.
- Browser print refinements may still be valid, but they should not be confused with exportable cross-target theme values.
