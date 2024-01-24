# Date

**A calendar date encoded as a ISO 8601 string.**

**`@id`**: [`schema:Date`](https://schema.org/Date)

## Properties

The `Date` type has these properties:

| Name    | Aliases | `@id`                                      | Type                                                                                            | Description                     | Inherited from                                                                                   |
| ------- | ------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`    | -       | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item.   | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `value` | -       | [`schema:value`](https://schema.org/value) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The date as an ISO 8601 string. | -                                                                                                |

## Related

The `Date` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: none

## Formats

The `Date` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding     | Decoding  | Status              | Notes                                                                                                                     |
| -------------------------------------------------------------------------------------------------- | ------------ | --------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| [DOM HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/dom.md)           | 🟢 No loss    |           | 🚧 Under development |                                                                                                                           |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss   |           | 🚧 Under development |                                                                                                                           |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              | 🟢 No loss    | 🟢 No loss | 🚧 Under development | Encoded as [`<date>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/date.html) using special function |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                                           |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss |           | ⚠️ Alpha            |                                                                                                                           |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                           |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                           |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss    | 🟢 No loss | 🔶 Beta              |                                                                                                                           |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                           |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                           |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss    | 🟢 No loss | 🟢 Stable            |                                                                                                                           |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss   |           | 🟢 Stable            |                                                                                                                           |

## Bindings

The `Date` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Date.jsonld)
- [JSON Schema](https://stencila.org/Date.schema.json)
- Python class [`Date`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/date.py)
- Rust struct [`Date`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/date.rs)
- TypeScript class [`Date`](https://github.com/stencila/stencila/blob/main/ts/src/types/Date.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `Date` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property | Complexity | Description                                                             | Strategy                              |
| -------- | ---------- | ----------------------------------------------------------------------- | ------------------------------------- |
| `value`  | Min+       | Generate a fixed date string.                                           | `String::from("2022-02-22")`          |
|          | Low+       | Generate a random date string.                                          | Regex `[0-9]{4}-[01][0-9]-[0-3][1-9]` |
|          | High+      | Generate a random string of up to 10 alphanumeric characters & hyphens. | Regex `[a-zA-Z0-9\-]{1,10}`           |
|          | Max        | Generate an arbitrary string.                                           | `String::arbitrary()`                 |

## Source

This documentation was generated from [`Date.yaml`](https://github.com/stencila/stencila/blob/main/schema/Date.yaml) by [`docs_type.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_type.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.