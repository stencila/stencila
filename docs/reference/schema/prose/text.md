# Text

**Textual content.**

Intended mostly for use for inline text e.g. the text in a paragraph.

Differs from the primitive `String` type in that it has a `type` and `id` property.
The `id` property allows use to identify text nodes with a sequence of inline nodes
for better diffing.

Also, in Rust, the `value` property is implemented as a CRDT.


**`@id`**: [`schema:Text`](https://schema.org/Text)

## Properties

The `Text` type has these properties:

| Name    | Aliases | `@id`                                      | Type                                                                                            | Description                   | Inherited from                                                                                   |
| ------- | ------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`    | -       | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item. | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `value` | -       | [`schema:value`](https://schema.org/value) | [`Cord`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)     | The value of the text content | -                                                                                                |

## Related

The `Text` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Text` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                               | Encoding   | Decoding  | Status              | Notes                                                                                 |
| ---------------------------------------------------------------------------------------------------- | ---------- | --------- | ------------------- | ------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.html.md)        | 🟢 No loss  |           | 🔶 Beta              |                                                                                       |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)                | 🟢 No loss  |           | 🚧 Under development | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span) |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)                | 🟢 No loss  | 🟢 No loss | 🚧 Under development | Encoded using special function                                                        |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)        | 🟢 No loss  | 🟢 No loss | 🔶 Beta              | Encoded using implemented function                                                    |
| [Stencila Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/smd.md)    | 🟢 No loss  | 🟢 No loss | 🔶 Beta              |                                                                                       |
| [MyST Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/myst.md)       | 🟢 No loss  | 🟢 No loss | 🔶 Beta              |                                                                                       |
| [LLM Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/llmd.md)        | 🟢 No loss  | 🟢 No loss | 🔶 Beta              |                                                                                       |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)          | 🟢 No loss  |           | 🔶 Beta              |                                                                                       |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)                | 🟢 No loss  | 🟢 No loss | 🟢 Stable            |                                                                                       |
| [JSON+Zip](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.zip.md)        | 🟢 No loss  | 🟢 No loss | 🟢 Stable            |                                                                                       |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)              | 🟢 No loss  | 🟢 No loss | 🟢 Stable            |                                                                                       |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)           | 🟢 No loss  | 🟢 No loss | 🔶 Beta              |                                                                                       |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)                | 🟢 No loss  | 🟢 No loss | 🟢 Stable            |                                                                                       |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.zstd.md) | 🟢 No loss  | 🟢 No loss | 🟢 Stable            |                                                                                       |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)                | 🟢 No loss  | 🟢 No loss | 🟢 Stable            |                                                                                       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/formats/directory.md)      |            |           | 🚧 Under development |                                                                                       |
| [Stencila Web Bundle](https://github.com/stencila/stencila/blob/main/docs/reference/formats/swb.md)  |            |           | ⚠️ Alpha            |                                                                                       |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)              | 🔷 Low loss |           | 🟢 Stable            |                                                                                       |

## Bindings

The `Text` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Text.jsonld)
- [JSON Schema](https://stencila.org/Text.schema.json)
- Python class [`Text`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/text.py)
- Rust struct [`Text`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/text.rs)
- TypeScript class [`Text`](https://github.com/stencila/stencila/blob/main/ts/src/types/Text.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Text` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                                                                                    | Strategy                                                        |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------- |
| `value`  | Min+       | Generate a fixed string of text.                                                                                               | `Cord::from("text")`                                            |
|          | Low+       | Generate a random string of up to 10 alphanumeric characters.                                                                  | `r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)`                     |
|          | High+      | Generate a random string of up to 100 alphanumeric characters, some special characters commonly used in prose, and whitespace. | `r"[a-zA-Z0-9 \t\-_.!?*+-/()'<>=]{1,100}".prop_map(Cord::from)` |
|          | Max        | Generate an arbitrary string.                                                                                                  | `String::arbitrary().prop_map(Cord::from)`                      |

## Source

This documentation was generated from [`Text.yaml`](https://github.com/stencila/stencila/blob/main/schema/Text.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
