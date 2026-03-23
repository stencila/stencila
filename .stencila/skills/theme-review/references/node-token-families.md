# Node token family review reference

Use this file when reviewing document-theme styling that goes beyond whole-document semantic choices.

For a comprehensive current list of builtin node tokens, use the CLI:

```sh
stencila themes tokens --scope node
```

Filter to a family when you need a narrower inventory:

```sh
stencila themes tokens --scope node --family heading
stencila themes tokens --scope node --family table
stencila themes tokens --scope node --family code
```

Use machine-readable output when you need to inspect or post-process the inventory:

```sh
stencila themes tokens --scope node --as json
```

## When node tokens are appropriate

Start with semantic tokens first. Node tokens are justified when the theme needs styling that applies to specific document structures rather than the whole document.

Typical examples:

- heading hierarchy beyond the global heading font family
- link treatment beyond the accent color
- code blocks or inline code with distinct styling
- table borders, zebra striping, cell spacing, or caption treatment
- figure, image, quote, admonition, citation, or reference-specific styling
- module-specific spacing, borders, backgrounds, or typography

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
- `break`
- `call`
- `claim`
- `discussion`
- `for`
- `if`
- `include`
- `instruction`
- `prompt`
- `review`
- `suggestion`
- `styled`

Treat this as a guidance list, not a guaranteed exhaustive inventory. Use the CLI for the exact current families and token names.

## Practical decision rule for review

Prefer semantic tokens when the desired styling should feel consistent across most of the document.

Prefer node tokens when one content type should stand apart, for example:

- quieter tables without changing all borders globally
- stronger heading spacing without changing body spacing everywhere
- branded admonitions without turning every surface into a callout
- distinct code-block treatment without changing general text styling

Flag cases where node tokens are used for effects that semantic tokens already cover.

## How to use the CLI and this reference together

1. Start by checking whether semantic tokens cover the theme's typography, colors, spacing, surfaces, and layout.
2. If the artifact targets a specific document structure, query `stencila themes tokens --scope node`.
3. Narrow with `--family <name>` when the relevant structure is clear.
4. Use the CLI output to verify exact token names.
5. Use this reference to decide whether a node-specific override is justified or whether the theme should stay at the semantic layer.

## Review cautions

- Prefer semantic tokens when the desired styling should feel consistent across most of the document.
- Prefer node tokens when one content type should stand apart.
- Do not assume every node family maps equally to PDF, DOCX, or email.
- If an exact node token is not verified through `stencila themes tokens --scope node`, report the uncertainty instead of approving the guess.
- Treat broad selector overrides as a fallback when token-based styling is unavailable or insufficient.
- Flag node-specific overrides that conflict with or duplicate semantic foundations.
