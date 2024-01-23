# Citation Mode

**The mode of a `Cite`.**

There are two main citation modes: parenthetical and narrative (a.k.a textual).
See https://apastyle.apa.org/style-grammar-guidelines/citations/basic-principles/parenthetical-versus-narrative
for an explanation.

This property is optional and tools are recommended to assume `parenthetical` if missing.

Narrative citations will usually be of form "As noted by Smith (1992)," but `narrative-author`
allows for "In the early nineties, Smith noted" and `narrative-year` allows for "As noted by Smith in 1992 and 1993".

Pandoc's `CitationMode` enumeration has `Normal` (for `parenthetical`), `AuthorInText` (for `textual`),
and `SuppressAuthor` (for `textual-year`).
See https://github.com/jgm/pandoc-types/blob/0158cd0e2a2ca9d6f14389a1a57bc64cab45a7dd/src/Text/Pandoc/Definition.hs#L353.

LaTeX's `natbib` package has `\citep{}` (for `parenthetical`), `\citet{}` (for `textual`),
`\citeauthor{}` (for `textual-author`), `\citeyear{}` (for `textual-year`).
See https://www.overleaf.com/learn/latex/Natbib_citation_styles.


**`@id`**: `stencila:CitationMode`

## Members

The `CitationMode` type has these members:

- `Parenthetical`
- `Narrative`
- `NarrativeAuthor`

## Bindings

The `CitationMode` type is represented in these bindings:

- [JSON-LD](https://stencila.org/CitationMode.jsonld)
- [JSON Schema](https://stencila.org/CitationMode.schema.json)
- Python type [`CitationMode`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/citation_mode.py)
- Rust type [`CitationMode`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation_mode.rs)
- TypeScript type [`CitationMode`](https://github.com/stencila/stencila/blob/main/ts/src/types/CitationMode.ts)

## Source

This documentation was generated from [`CitationMode.yaml`](https://github.com/stencila/stencila/blob/main/schema/CitationMode.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).