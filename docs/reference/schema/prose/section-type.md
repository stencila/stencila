# Section Type

**The type of a `Section`.**

**`@id`**: `stencila:SectionType`

## Members

The `SectionType` type has these members:

- `Main`
- `Header`
- `Footer`
- `Summary`
- `Introduction`
- `Methods`
- `Results`
- `Discussion`
- `Conclusion`

## Bindings

The `SectionType` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/SectionType.jsonld)
- [JSON Schema](https://stencila.dev/SectionType.schema.json)
- Python type [`SectionType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/section_type.py)
- Rust type [`SectionType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/section_type.rs)
- TypeScript type [`SectionType`](https://github.com/stencila/stencila/blob/main/typescript/src/types/SectionType.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `SectionType` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

|         |            |             |          |
| ------- | ---------- | ----------- | -------- |
| Variant | Complexity | Description | Strategy |

## Source

This documentation was generated from [`SectionType.yaml`](https://github.com/stencila/stencila/blob/main/schema/SectionType.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).