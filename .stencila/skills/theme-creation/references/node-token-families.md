# Node token family reference

Use this file for document-theme guidance on node-specific token families, when to move beyond semantic tokens, and how to interpret `--scope node` results from the theme CLI.

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

## When to use node tokens

Start with semantic tokens first. Move to node tokens when the user wants styling that applies to specific document structures rather than the whole document.

Typical examples:

- heading hierarchy beyond the global heading font family
- link treatment beyond the accent color
- code blocks or inline code that need distinct styling
- table borders, zebra striping, cell spacing, or caption treatment
- figure, image, quote, admonition, citation, or reference-specific styling
- module-specific spacing, borders, backgrounds, or typography

## Common node families to look for

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

## How to use the CLI and this reference together

1. Start with semantic tokens for typography, colors, spacing, surfaces, and layout.
2. If the request targets a specific document structure, query `stencila themes tokens --scope node`.
3. Narrow with `--family <name>` when the relevant structure is clear.
4. Use the CLI output to pick exact token names.
5. Use this reference to decide whether a node-specific override is appropriate or whether the theme should stay at the semantic layer.

## Practical decision rule

Prefer semantic tokens when the desired styling should feel consistent across most of the document.

Prefer node tokens when the user wants one content type to stand apart, for example:

- quieter tables without changing all borders globally
- stronger heading spacing without changing body spacing everywhere
- branded admonitions without turning every surface into a callout
- distinct code-block treatment without changing general text styling

## Authoring cautions

- Keep exported cross-target choices at top-level `:root` when they need to influence non-web outputs.
- Do not assume every node family maps equally to PDF, DOCX, or email.
- Use focused selectors only when tokens are insufficient.
- If the user asks for a token that you cannot verify via `stencila themes tokens --scope node`, describe the family and intended effect instead of guessing.

## Validation workflow

When document-node styling matters, recommend checks such as:

1. inspect the affected node types in HTML output
2. confirm the targeted structures actually use the intended tokens
3. compare PDF or DOCX output separately if those targets matter
4. ensure node-specific overrides do not unintentionally conflict with semantic foundations
5. run `stencila themes validate theme.css` after editing the file; use `--strict` if unknown tokens should fail

## Verified source basis

Localized from the Stencila theme architecture and document-node token documentation when this skill was prepared.
