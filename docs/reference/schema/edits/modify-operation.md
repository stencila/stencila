# Modify Operation

**An operation that is part of a suggestion to modify the property of a node.**

**`@id`**: `stencila:ModifyOperation`

## Properties

The `ModifyOperation` type has these properties:

| Name     | Aliases | `@id`                                        | Type                                                                                                                                                                                                                 | Description                                                      | Inherited from                                                                                   |
| -------- | ------- | -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`     | -       | [`schema:id`](https://schema.org/id)         | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The identifier for this item.                                    | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `target` | -       | [`schema:target`](https://schema.org/target) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                      | The target property of each node to be modified.                 | -                                                                                                |
| `value`  | -       | [`schema:value`](https://schema.org/value)   | [`StringPatch`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/string-patch.md) \| [`Primitive`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/primitive.md) | The new value, or string patch, to apply to the target property. | -                                                                                                |

## Related

The `ModifyOperation` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `ModifyOperation` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ----- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |       |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |              |           | 🚧 Under development |       |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |       |

## Bindings

The `ModifyOperation` type is represented in these bindings:

- [JSON-LD](https://stencila.org/ModifyOperation.jsonld)
- [JSON Schema](https://stencila.org/ModifyOperation.schema.json)
- Python class [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/modify_operation.py)
- Rust struct [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/modify_operation.rs)
- TypeScript class [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/ts/src/types/ModifyOperation.ts)

## Source

This documentation was generated from [`ModifyOperation.yaml`](https://github.com/stencila/stencila/blob/main/schema/ModifyOperation.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).