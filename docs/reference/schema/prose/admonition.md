# Admonition

**A admonition within a document.**

**`@id`**: `stencila:Admonition`

## Properties

The `Admonition` type has these properties:

| Name             | Aliases                              | `@id`                                            | Type                                                                                                                 | Description                                                       | Inherited from                                                                                   |
| ---------------- | ------------------------------------ | ------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`             | -                                    | [`schema:id`](https://schema.org/id)             | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                      | The identifier for this item.                                     | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `admonitionType` | `admonition-type`, `admonition_type` | `stencila:admonitionType`                        | [`AdmonitionType`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition-type.md)    | The type of admonition.                                           | -                                                                                                |
| `title`          | -                                    | [`schema:headline`](https://schema.org/headline) | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                    | The title of the admonition.                                      | -                                                                                                |
| `isFolded`       | `is-folded`, `is_folded`             | `stencila:isFolded`                              | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)                    | Whether the admonition is folded.                                 | -                                                                                                |
| `content`        | -                                    | `stencila:content`                               | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*                      | The content within the section.                                   | -                                                                                                |
| `authors`        | `author`                             | [`schema:author`](https://schema.org/author)     | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                    | The authors of the admonition.                                    | -                                                                                                |
| `provenance`     | -                                    | `stencila:provenance`                            | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)* | A summary of the provenance of the content within the admonition. | -                                                                                                |

## Related

The `Admonition` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Admonition` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                                                                                                          |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | -------------------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                                                                                                |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development | Encoded as [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)                        |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss    | 🟢 No loss | 🚧 Under development | Encoded as [`<boxed-text>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/boxed-text.html) |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | 🟢 No loss    | 🟢 No loss | ⚠️ Alpha            | Encoded using implemented function                                                                             |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | 🟢 No loss    | 🟢 No loss | ⚠️ Alpha            |                                                                                                                |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                                |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                                                                                                |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                                                                                                |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                                                                                                |

## Bindings

The `Admonition` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Admonition.jsonld)
- [JSON Schema](https://stencila.org/Admonition.schema.json)
- Python class [`Admonition`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/admonition.py)
- Rust struct [`Admonition`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/admonition.rs)
- TypeScript class [`Admonition`](https://github.com/stencila/stencila/blob/main/ts/src/types/Admonition.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Admonition` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property         | Complexity | Description                                                 | Strategy                                   |
| ---------------- | ---------- | ----------------------------------------------------------- | ------------------------------------------ |
| `admonitionType` | Min+       | Fixed admonition type.                                      | `AdmonitionType::Info`                     |
|                  | Low+       | Generate an arbitrary admonition type.                      | `AdmonitionType::arbitrary()`              |
| `title`          | Min+       | No title.                                                   | `None`                                     |
|                  | Low+       | Generate up to two arbitrary, non-recursive, inline nodes.  | `option::of(vec_inlines_non_recursive(2))` |
|                  | High+      | Generate up to four arbitrary, non-recursive, inline nodes. | `option::of(vec_inlines_non_recursive(4))` |
| `isFolded`       | Min+       | Not foldable.                                               | `None`                                     |
|                  | Low+       | Arbitrarily, un-foldable, folded, or unfolded.              | `option::of(bool::arbitrary())`            |
| `content`        | Min+       | A single, simple paragraph.                                 | `vec![p([t("Admonition content")])]`       |
|                  | Low+       | Generate up to two arbitrary, non-recursive, block nodes.   | `vec_blocks_non_recursive(2)`              |
|                  | High+      | Generate up to four arbitrary, non-recursive, block nodes.  | `vec_blocks_non_recursive(4)`              |

## Source

This documentation was generated from [`Admonition.yaml`](https://github.com/stencila/stencila/blob/main/schema/Admonition.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
