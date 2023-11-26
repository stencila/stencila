# Instruct

**Abstract base type for a document editing instruction.**

**`@id`**: `stencila:Instruct`

## Properties

The `Instruct` type has these properties:

| Name              | Aliases                                | `@id`                                      | Type                                                                                                                                                                                                                                                                                                                                          | Description                                 | Inherited from                                                                                   |
| ----------------- | -------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| `id`              | -                                      | [`schema:id`](https://schema.org/id)       | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                                                                                                                                               | The identifier for this item.               | [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md) |
| `text`            | -                                      | [`schema:text`](https://schema.org/text)   | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                                                                                                                                                                                                                                               | The text of the instruction.                | -                                                                                                |
| `agent`           | -                                      | [`schema:agent`](https://schema.org/agent) | [`Person`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md) \| [`Organization`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md) \| [`SoftwareApplication`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md) | The agent that executed the instruction.    | -                                                                                                |
| `executionStatus` | `execution-status`, `execution_status` | `stencila:executionStatus`                 | [`ExecutionStatus`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-status.md)                                                                                                                                                                                                                            | Status of the execution of the instruction. | -                                                                                                |

## Related

The `Instruct` type is related to these types:

- Parents: [`Entity`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md)
- Children: [`InstructBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruct-block.md), [`InstructInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruct-inline.md)

## Bindings

The `Instruct` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Instruct.jsonld)
- [JSON Schema](https://stencila.org/Instruct.schema.json)
- Python class [`Instruct`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/instruct.py)
- Rust struct [`Instruct`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/instruct.rs)
- TypeScript class [`Instruct`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Instruct.ts)

## Source

This documentation was generated from [`Instruct.yaml`](https://github.com/stencila/stencila/blob/main/schema/Instruct.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).