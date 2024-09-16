# Suggestion Inline

**Abstract base type for nodes that indicate a suggested change to inline content.**

**`@id`**: `stencila:SuggestionInline`

## Properties

The `SuggestionInline` type has these properties:

| Name                | Aliases                                    | `@id`                                        | Type                                                                                                                  | Description                                                                           | Inherited from                                                                                           |
| ------------------- | ------------------------------------------ | -------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `id`                | -                                          | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                       | The identifier for this item.                                                         | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)         |
| `suggestionStatus`  | `suggestion-status`, `suggestion_status`   | `stencila:suggestionStatus`                  | [`SuggestionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-status.md) | The status of the suggestion including whether it is proposed, accepted, or rejected. | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md) |
| `authors`           | `author`                                   | [`schema:author`](https://schema.org/author) | [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)*                     | The authors of the suggestion                                                         | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md) |
| `provenance`        | -                                          | `stencila:provenance`                        | [`ProvenanceCount`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance-count.md)*  | A summary of the provenance of the content within the suggestion.                     | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md) |
| `executionDuration` | `execution-duration`, `execution_duration` | `stencila:executionDuration`                 | [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                   | Time taken to generate the suggestion.                                                | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md) |
| `executionEnded`    | `execution-ended`, `execution_ended`       | `stencila:executionEnded`                    | [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                 | The timestamp when the generation ended.                                              | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md) |
| `feedback`          | -                                          | `stencila:feedback`                          | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                       | Feedback on the suggestion                                                            | [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md) |
| `content`           | -                                          | `stencila:content`                           | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                     | The content that is suggested to be inserted, modified, replaced, or deleted.         | -                                                                                                        |

## Related

The `SuggestionInline` type is related to these types:

- Parents: [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md)
- Children: [`DeleteInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete-inline.md), [`InsertInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert-inline.md), [`ModifyInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-inline.md), [`ReplaceInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/replace-inline.md)

## Formats

The `SuggestionInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🚧 Under development |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | ⚠️ Alpha            | Encoded using implemented function |
| [MyST](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)                | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | ⚠️ Alpha            |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |           | 🚧 Under development |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                    |

## Bindings

The `SuggestionInline` type is represented in these bindings:

- [JSON-LD](https://stencila.org/SuggestionInline.jsonld)
- [JSON Schema](https://stencila.org/SuggestionInline.schema.json)
- Python class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/suggestion_inline.py)
- Rust struct [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/suggestion_inline.rs)
- TypeScript class [`SuggestionInline`](https://github.com/stencila/stencila/blob/main/ts/src/types/SuggestionInline.ts)

## Source

This documentation was generated from [`SuggestionInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/SuggestionInline.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
