# Node token family review reference

Use this file when reviewing document-theme styling that goes beyond whole-document semantic choices.

## When node tokens are appropriate

Start with semantic tokens first. Node tokens are justified when the theme needs styling that applies to specific document structures rather than the whole document.

Typical examples:

- heading hierarchy beyond the global heading font family
- link treatment beyond the accent color
- code blocks or inline code with distinct styling
- table borders, zebra striping, cell spacing, or caption treatment
- figure, image, quote, admonition, citation, or reference-specific styling

## Common node families to verify

Use `stencila themes tokens --scope node` to inspect exact names in families such as:

- `heading`
- `paragraph`
- `list`
- `link`
- `code`
- `table`
- `datatable`
- `figure`
- `image`
- `quote`
- `math`
- `admonition`
- `citation`
- `references`
- `works`

## Review cautions

- Prefer semantic tokens when the desired styling should feel consistent across most of the document.
- Prefer node tokens when one content type should stand apart.
- Do not assume every node family maps equally to PDF, DOCX, or email.
- If an exact node token is not verified through `stencila themes tokens --scope node`, report the uncertainty instead of approving the guess.
- Treat broad selector overrides as a fallback when token-based styling is unavailable or insufficient.
