---
title: Defined Term
description: A word, name, acronym, phrase, etc. with a formal definition.
---

Often used in the context of category or subject classification,  glossaries or dictionaries, product or creative work types, etc.
Use the `name` property for the term being defined, use `termCode`. If the term has an alpha-numeric code allocated, use
description to provide the definition of the term.


# Properties

The `DefinedTerm` type has these properties:

| Name             | Description                                                     | Type                                                                 | Inherited from          |
| ---------------- | --------------------------------------------------------------- | -------------------------------------------------------------------- | ----------------------- |
| `id`             | The identifier for this item.                                   | [`String`](./string.md)                                              | [`Entity`](./entity.md) |
| `alternateNames` | Alternate names (aliases) for the item.                         | [`String`](./string.md)*                                             | [`Thing`](./thing.md)   |
| `description`    | A description of the item.                                      | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `identifiers`    | Any kind of identifier for any kind of Thing.                   | ([`PropertyValue`](./property-value.md) \| [`String`](./string.md))* | [`Thing`](./thing.md)   |
| `images`         | Images of the item.                                             | [`ImageObject`](./image-object.md)*                                  | [`Thing`](./thing.md)   |
| `name`           | The name of the item.                                           | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `url`            | The URL of the item.                                            | [`String`](./string.md)                                              | [`Thing`](./thing.md)   |
| `termCode`       | A code that identifies this DefinedTerm within a DefinedTermSet | [`String`](./string.md)                                              | -                       |

# Related

The `DefinedTerm` type is related to these types:

- Parents: [`Thing`](./thing.md)
- Children: none

# Bindings

The `DefinedTerm` type is represented in:

- [JSON-LD](https://stencila.org/DefinedTerm.jsonld)
- [JSON Schema](https://stencila.org/DefinedTerm.schema.json)
- Python class [`DefinedTerm`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`DefinedTerm`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/defined_term.rs)
- TypeScript class [`DefinedTerm`](https://github.com/stencila/stencila/blob/main/ts/src/types/DefinedTerm.ts)

***

This documentation was generated from [`DefinedTerm.yaml`](https://github.com/stencila/stencila/blob/main/schema/DefinedTerm.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
