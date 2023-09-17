---
title:
- type: Text
  value: Variable
---

# Variable

**A variable representing a name / value pair.**

**`@id`**: `stencila:Variable`

This type is marked as experimental and is likely to change.

## Properties

The `Variable` type has these properties:

| Name      | `@id`                                      | Type                                                               | Description                                                               | Inherited from                                                         |
| --------- | ------------------------------------------ | ------------------------------------------------------------------ | ------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| id        | [`schema:id`](https://schema.org/id)       | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item                                              | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)    |
| namespace | `stencila:namespace`                       | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The namespace, usually a document path, within which the variable resides | [`Variable`](https://stencila.dev/docs/reference/schema/flow/variable) |
| name      | [`schema:name`](https://schema.org/name)   | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The name of the variable.                                                 | [`Variable`](https://stencila.dev/docs/reference/schema/flow/variable) |
| kind      | `stencila:kind`                            | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`     | [`Variable`](https://stencila.dev/docs/reference/schema/flow/variable) |
| value     | [`schema:value`](https://schema.org/value) | [`Node`](https://stencila.dev/docs/reference/schema/other/node)    | The value of the variable.                                                | [`Variable`](https://stencila.dev/docs/reference/schema/flow/variable) |

## Related

The `Variable` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `Variable` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                           | Encoding       | Decoding     | Status                 | Notes |
| ---------------------------------------------------------------- | -------------- | ------------ | ---------------------- | ----- |
| [HTML](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [JATS](https://stencila.dev/docs/reference/formats/{name})       | 🔷 Low loss     |              | 🚧 Under development    |       |
| [Markdown](https://stencila.dev/docs/reference/formats/{name})   | 🟥 High loss    |              | 🚧 Under development    |       |
| [Plain text](https://stencila.dev/docs/reference/formats/{name}) | 🟥 High loss    |              | 🟥 Alpha                |       |
| [JSON](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [JSON5](https://stencila.dev/docs/reference/formats/{name})      | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [YAML](https://stencila.dev/docs/reference/formats/{name})       | 🟢 No loss      | 🟢 No loss    | 🟢 Stable               |       |
| [Debug](https://stencila.dev/docs/reference/formats/{name})      | 🔷 Low loss     |              | 🟢 Stable               |       |

## Bindings

The `Variable` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Variable.jsonld)
- [JSON Schema](https://stencila.dev/Variable.schema.json)
- Python class [`Variable`](https://github.com/stencila/stencila/blob/main/python/stencila/types/variable.py)
- Rust struct [`Variable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/variable.rs)
- TypeScript class [`Variable`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Variable.ts)

## Source

This documentation was generated from [`Variable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Variable.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).