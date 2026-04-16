---
title: Citation Mode
description: The presentation mode of a citation.
---

This is an enumeration used in Stencila Schema for citation presentation modes.

It exists so citations can preserve the distinction between parenthetical and
narrative forms, including variants that expose only the author or only the
year in surrounding prose. This helps Stencila map citation semantics
consistently across authoring formats, renderers, and citation processors.

See [`Citation.citationMode`](./citation.md#citationmode) for the property
that uses this enumeration.


# Analogues

The following external types, elements, or nodes are similar to a `CitationMode`:

- [CSL citation position and narrative conventions](https://docs.citationstyles.org/en/stable/specification.html): Close analogue for distinctions such as parenthetical versus narrative citation rendering, though Stencila exposes them as a compact schema enumeration.

# Members

The `CitationMode` type has these members:

| Member            | Description |
| ----------------- | ----------- |
| `Parenthetical`   | -           |
| `Narrative`       | -           |
| `NarrativeAuthor` | -           |
| `NarrativeYear`   | -           |

# Bindings

The `CitationMode` type is represented in:

- [JSON-LD](https://stencila.org/CitationMode.jsonld)
- [JSON Schema](https://stencila.org/CitationMode.schema.json)
- Python type [`CitationMode`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`CitationMode`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation_mode.rs)
- TypeScript type [`CitationMode`](https://github.com/stencila/stencila/blob/main/ts/src/types/CitationMode.ts)

***

This documentation was generated from [`CitationMode.yaml`](https://github.com/stencila/stencila/blob/main/schema/CitationMode.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
