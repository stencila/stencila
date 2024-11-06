# Walkthrough Step

**A step in a walkthrough.**

**`@id`**: `stencila:WalkthroughStep`

This type is marked as unstable and is subject to change.

## Properties

The `WalkthroughStep` type has these properties:

| Name          | Aliases                        | `@id`                                | Type                                                                                              | Description                                                                      | Inherited from                                                                                   |
| ------------- | ------------------------------ | ------------------------------------ | ------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`          | -                              | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)   | The identifier for this item.                                                    | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `isCollapsed` | `is-collapsed`, `is_collapsed` | `stencila:isCollapsed`               | [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md) | Whether this step is active (i.e. is encoded in source format and can be edited) | -                                                                                                |
| `content`     | -                              | `stencila:content`                   | [`Block`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/block.md)*   | The content of the step.                                                         | -                                                                                                |

## Related

The `WalkthroughStep` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `WalkthroughStep` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding     | Decoding  | Status              | Notes                              |
| ---------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ---------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss    |           | 🔶 Beta              |                                    |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🔷 Low loss   |           | 🚧 Under development |                                    |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                |              |           | 🚧 Under development |                                    |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | ⚠️ High loss |           | 🔶 Beta              | Encoded using implemented function |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | ⚠️ High loss |           | 🔶 Beta              |                                    |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | ⚠️ High loss |           | 🔶 Beta              |                                    |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | ⚠️ High loss |           | 🔶 Beta              |                                    |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | ⚠️ High loss |           | 🔶 Beta              |                                    |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                    |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                    |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |              |           | 🚧 Under development |                                    |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |              |           | ⚠️ Alpha            |                                    |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss   |           | 🟢 Stable            |                                    |

## Bindings

The `WalkthroughStep` type is represented in these bindings:

- [JSON-LD](https://stencila.org/WalkthroughStep.jsonld)
- [JSON Schema](https://stencila.org/WalkthroughStep.schema.json)
- Python class [`WalkthroughStep`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/walkthrough_step.py)
- Rust struct [`WalkthroughStep`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/walkthrough_step.rs)
- TypeScript class [`WalkthroughStep`](https://github.com/stencila/stencila/blob/main/ts/src/types/WalkthroughStep.ts)

## Source

This documentation was generated from [`WalkthroughStep.yaml`](https://github.com/stencila/stencila/blob/main/schema/WalkthroughStep.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).
