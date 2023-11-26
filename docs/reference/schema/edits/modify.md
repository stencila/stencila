# Modify

**A suggestion to modify one or more nodes.**

**`@id`**: `stencila:Modify`

## Properties

The `Modify` type has these properties:

| Name         | Aliases     | `@id`                 | Type                                                                                                                 | Description                                | Inherited from |
| ------------ | ----------- | --------------------- | -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ | -------------- |
| `operations` | `operation` | `stencila:operations` | [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-operation.md)* | The operations to be applied to the nodes. | -              |

## Related

The `Modify` type is related to these types:

- Parents: none
- Children: [`ModifyBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-block.md), [`ModifyInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-inline.md)

## Bindings

The `Modify` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Modify.jsonld)
- [JSON Schema](https://stencila.org/Modify.schema.json)
- Python class [`Modify`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/modify.py)
- Rust struct [`Modify`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/modify.rs)
- TypeScript class [`Modify`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Modify.ts)

## Source

This documentation was generated from [`Modify.yaml`](https://github.com/stencila/stencila/blob/main/schema/Modify.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).