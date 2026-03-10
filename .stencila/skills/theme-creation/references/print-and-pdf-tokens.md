# Print and PDF token reference

Use this file for print and PDF workflow guidance, page-token categories, and caveats about how paged tokens are applied.

For a comprehensive current list of builtin print tokens, use the CLI:

```sh
stencila themes tokens --scope print
```

Use machine-readable output when you need to inspect or post-process the inventory:

```sh
stencila themes tokens --scope print --as json
```

## Start here

For paged output, begin with token groups such as:

- page size tokens
- page margin tokens
- running header and footer content tokens
- margin-box typography tokens
- page border tokens
- first-page override tokens

Keep these at top-level `:root` if they need to be exported into non-web targets.

## Important token groups

Use the CLI listing to find exact names in groups such as:

- `--page-width` and `--page-height`
- `--page-margin-*`
- `--page-top-*-content`
- `--page-bottom-*-content`
- `--page-margin-font-*`
- `--page-border-*`
- `--page1-*`

## How print tokens are applied

- PDF is produced from HTML with print media enabled.
- Page tokens control page size, margins, headers, footers, and related print presentation.
- Margin box content can reference injected metadata variables such as `--document-title`, `--document-authors`, `--document-doi`, and `--document-date`.
- Keep exportable page tokens in top-level `:root`; tokens nested inside `@media` or `@supports` are useful for browser print rendering but are not exported as general non-web theme values.

## How to use the CLI and this reference together

1. Use `stencila themes tokens --scope print` to get the current token inventory.
2. Pick the exact page and margin token names from the CLI output.
3. Use this reference to decide which token groups to change first and how those changes affect paged outputs.
4. Validate in print preview or rendered PDF because PDF and DOCX page mappings are not identical.

## Validation workflow

When the theme must support print or PDF, recommend checks such as:

1. render a PDF or print preview
2. verify page size and margins
3. compare running headers and footers across regular pages and the first page
4. confirm table, figure, and code blocks still fit the page width
5. re-check DOCX separately if page-layout parity matters, because PDF and DOCX mappings are not identical
6. run `stencila themes validate theme.css` after editing the file; use `--strict` if unknown tokens should fail

## Verified source basis

Localized from the Stencila print token documentation when this skill was prepared.
