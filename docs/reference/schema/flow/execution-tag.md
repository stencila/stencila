---
title:
- type: Text
  value: ExecutionTag
---

# Execution Tag

**A tag on code that affects its execution**

**`@id`**: `stencila:ExecutionTag`

## Properties

The `ExecutionTag` type has these properties:

| Name     | `@id`                                      | Type                                                                 | Description                               | Inherited from                                                                  |
| -------- | ------------------------------------------ | -------------------------------------------------------------------- | ----------------------------------------- | ------------------------------------------------------------------------------- |
| id       | [`schema:id`](https://schema.org/id)       | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The identifier for this item              | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)             |
| name     | [`schema:name`](https://schema.org/name)   | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The name of the tag                       | [`ExecutionTag`](https://stencila.dev/docs/reference/schema/flow/execution-tag) |
| value    | [`schema:value`](https://schema.org/value) | [`String`](https://stencila.dev/docs/reference/schema/data/string)   | The value of the tag                      | [`ExecutionTag`](https://stencila.dev/docs/reference/schema/flow/execution-tag) |
| isGlobal | `stencila:isGlobal`                        | [`Boolean`](https://stencila.dev/docs/reference/schema/data/boolean) | Whether the tag is global to the document | [`ExecutionTag`](https://stencila.dev/docs/reference/schema/flow/execution-tag) |

## Related

The `ExecutionTag` type is related to these types:

- Parents: [`Entity`](https://stencila.dev/docs/reference/schema/other/entity)
- Children: none

## Formats

The `ExecutionTag` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

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

The `ExecutionTag` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ExecutionTag.jsonld)
- [JSON Schema](https://stencila.dev/ExecutionTag.schema.json)
- Python class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/python/stencila/types/execution_tag.py)
- Rust struct [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/execution_tag.rs)
- TypeScript class [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ExecutionTag.ts)

## Source

This documentation was generated from [`ExecutionTag.yaml`](https://github.com/stencila/stencila/blob/main/schema/ExecutionTag.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).