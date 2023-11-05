# Admonition Type

**The type of an `Admonition`.**

**`@id`**: `stencila:AdmonitionType`

## Members

The `AdmonitionType` type has these members:

- `Note`
- `Info`
- `Tip`
- `Important`
- `Success`
- `Failure`
- `Warning`
- `Danger`
- `Error`

## Bindings

The `AdmonitionType` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/AdmonitionType.jsonld)
- [JSON Schema](https://stencila.dev/AdmonitionType.schema.json)
- Python type [`AdmonitionType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/admonition_type.py)
- Rust type [`AdmonitionType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/admonition_type.rs)
- TypeScript type [`AdmonitionType`](https://github.com/stencila/stencila/blob/main/typescript/src/types/AdmonitionType.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `AdmonitionType` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

|         |            |             |          |
| ------- | ---------- | ----------- | -------- |
| Variant | Complexity | Description | Strategy |

## Source

This documentation was generated from [`AdmonitionType.yaml`](https://github.com/stencila/stencila/blob/main/schema/AdmonitionType.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).