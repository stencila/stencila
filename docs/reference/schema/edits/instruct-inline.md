# Instruct Inline

**An instruction to edit some inline content.**

**`@id`**: `stencila:InstructInline`

## Properties

The `InstructInline` type has these properties:

| Name              | Aliases                                | `@id`                                      | Type                                                                                                                                                                                                                                                                                                                                          | Description                                   | Inherited from                                                                                       |
| ----------------- | -------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `id`              | -                                      | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                                                                                                                                               | The identifier for this item.                 | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)     |
| `text`            | -                                      | [`schema:text`](https://schema.org/text)   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                                                                                                                                               | The text of the instruction.                  | [`Instruct`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruct.md) |
| `agent`           | -                                      | [`schema:agent`](https://schema.org/agent) | [`Person`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md) \| [`Organization`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md) \| [`SoftwareApplication`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md) | The agent that executed the instruction.      | [`Instruct`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruct.md) |
| `executionStatus` | `execution-status`, `execution_status` | `stencila:executionStatus`                 | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)                                                                                                                                                                                                                            | Status of the execution of the instruction.   | [`Instruct`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruct.md) |
| `content`         | -                                      | `stencila:content`                         | [`Inline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/inline.md)*                                                                                                                                                                                                                                             | The content to which the instruction applies. | -                                                                                                    |

## Related

The `InstructInline` type is related to these types:

- Parents: [`Instruct`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruct.md)
- Children: none

## Formats

The `InstructInline` type can be encoded (serialized) to, and/or decoded (deserialized) from, these formats:

| Format                                                                                             | Encoding         | Decoding     | Status                 | Notes                          |
| -------------------------------------------------------------------------------------------------- | ---------------- | ------------ | ---------------------- | ------------------------------ |
| [HTML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/html.md)              | 🔷 Low loss       |              | 🚧 Under development    |                                |
| [JATS](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jats.md)              |                  |              | 🚧 Under development    |                                |
| [Markdown](https://github.com/stencila/stencila/blob/main/docs/reference/formats/markdown.md)      | ⚠️ High loss     |              | ⚠️ Alpha               | Encoded using special function |
| [Plain text](https://github.com/stencila/stencila/blob/main/docs/reference/formats/text.md)        | ⚠️ High loss     |              | ⚠️ Alpha               |                                |
| [JSON](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [JSON5](https://github.com/stencila/stencila/blob/main/docs/reference/formats/json5.md)            | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [JSON-LD](https://github.com/stencila/stencila/blob/main/docs/reference/formats/jsonld.md)         | 🟢 No loss        | 🟢 No loss    | 🔶 Beta                 |                                |
| [CBOR](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cbor.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [CBOR+Zstandard](https://github.com/stencila/stencila/blob/main/docs/reference/formats/cborzst.md) | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [YAML](https://github.com/stencila/stencila/blob/main/docs/reference/formats/yaml.md)              | 🟢 No loss        | 🟢 No loss    | 🟢 Stable               |                                |
| [Debug](https://github.com/stencila/stencila/blob/main/docs/reference/formats/debug.md)            | 🔷 Low loss       |              | 🟢 Stable               |                                |

## Bindings

The `InstructInline` type is represented in these bindings:

- [JSON-LD](https://stencila.org/InstructInline.jsonld)
- [JSON Schema](https://stencila.org/InstructInline.schema.json)
- Python class [`InstructInline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruct_inline.py)
- Rust struct [`InstructInline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruct_inline.rs)
- TypeScript class [`InstructInline`](https://github.com/stencila/stencila/blob/main/typescript/src/types/InstructInline.ts)

## Testing

During property-based (a.k.a generative) testing, the properties of the `InstructInline` type are generated using the following strategies[^1] for each complexity level. Any optional properties that are not in this table are set to `None`.

| Property  | Complexity | Description                                                | Strategy                                   |
| --------- | ---------- | ---------------------------------------------------------- | ------------------------------------------ |
| `content` | Min+       | No content                                                 | `None`                                     |
|           | Low+       | Generate a single arbitrary, non-recursive, inline node    | `option::of(vec_inlines_non_recursive(1))` |
|           | High+      | Generate up to two arbitrary, non-recursive, inline nodes  | `option::of(vec_inlines_non_recursive(2))` |
|           | Max        | Generate up to four arbitrary, non-recursive, inline nodes | `option::of(vec_inlines_non_recursive(4))` |

## Source

This documentation was generated from [`InstructInline.yaml`](https://github.com/stencila/stencila/blob/main/schema/InstructInline.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.