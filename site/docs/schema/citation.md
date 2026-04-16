---
title: Citation
description: A reference to a `CreativeWork` that is cited in another `CreativeWork`.
---

This is a type used in Stencila Schema for in-text citations of creative works.

It exists to represent a citation as a structured inline node that can be
resolved, rendered according to a document's citation style, and enriched with
rhetorical intent and pagination metadata. Unlike a plain text reference, it
can preserve both the citation target and its rendered form.

Key properties include `target`, `cites`, `citationMode`,
`citationIntent`, `content`, and any page or suffix metadata.


# Analogues

The following external types, elements, or nodes are similar to a `Citation`:

- JATS [`<xref>`](https://jats.nlm.nih.gov/archiving/tag-library/1.2/element/xref.html): Closest JATS cross-reference analogue for in-text citations, though Stencila citations carry richer citation-specific metadata and resolved reference data.
- MyST role [`cite`](https://mystmd.org/guide/roles#role-cite): Close MyST analogue for citation syntax in authoring, though Stencila stores the citation as a resolved inline node with additional metadata.

# Properties

The `Citation` type has these properties:

| Name                  | Description                                                                                           | Type                                                 | Inherited from          |
| --------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------------------- | ----------------------- |
| `target`              | The target of the citation (URL or reference ID).                                                     | [`String`](./string.md)                              | -                       |
| `compilationMessages` | Messages generated while resolving the target if the citation.                                        | [`CompilationMessage`](./compilation-message.md)*    | -                       |
| `cites`               | The `Reference` being cited, resolved from the `target`.                                              | [`Reference`](./reference.md)                        | -                       |
| `citationMode`        | Determines how the citation is shown within the surrounding text.                                     | [`CitationMode`](./citation-mode.md)                 | -                       |
| `citationIntent`      | The type/s of the citation, both factually and rhetorically.                                          | [`CitationIntent`](./citation-intent.md)*            | -                       |
| `content`             | A rendering of the citation using the citation style of the document.                                 | [`Inline`](./inline.md)*                             | -                       |
| `pageStart`           | The page on which the work starts; for example "135" or "xiii".                                       | [`Integer`](./integer.md) \| [`String`](./string.md) | -                       |
| `pageEnd`             | The page on which the work ends; for example "138" or "xvi".                                          | [`Integer`](./integer.md) \| [`String`](./string.md) | -                       |
| `pagination`          | Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55". | [`String`](./string.md)                              | -                       |
| `citationPrefix`      | Text to show before the citation.                                                                     | [`String`](./string.md)                              | -                       |
| `citationSuffix`      | Text to show after the citation.                                                                      | [`String`](./string.md)                              | -                       |
| `id`                  | The identifier for this item.                                                                         | [`String`](./string.md)                              | [`Entity`](./entity.md) |

# Related

The `Citation` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Citation` type is represented in:

- [JSON-LD](https://stencila.org/Citation.jsonld)
- [JSON Schema](https://stencila.org/Citation.schema.json)
- Python class [`Citation`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Citation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/citation.rs)
- TypeScript class [`Citation`](https://github.com/stencila/stencila/blob/main/ts/src/types/Citation.ts)

***

This documentation was generated from [`Citation.yaml`](https://github.com/stencila/stencila/blob/main/schema/Citation.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
