# Time Unit

**A unit in which time can be measured**

**`@id`**: `stencila:TimeUnit`

## Members

The `TimeUnit` type has these members:

- `Year`
- `Month`
- `Week`
- `Day`
- `Hour`
- `Minute`
- `Second`
- `Millisecond`
- `Microsecond`
- `Nanosecond`
- `Picosecond`
- `Femtosecond`
- `Attosecond`

## Bindings

The `TimeUnit` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/TimeUnit.jsonld)
- [JSON Schema](https://stencila.dev/TimeUnit.schema.json)
- Python type [`TimeUnit`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/time_unit.py)
- Rust type [`TimeUnit`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time_unit.rs)
- TypeScript type [`TimeUnit`](https://github.com/stencila/stencila/blob/main/typescript/src/types/TimeUnit.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `TimeUnit` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

|         |            |             |          |
| ------- | ---------- | ----------- | -------- |
| Variant | Complexity | Description | Strategy |

## Source

This documentation was generated from [`TimeUnit.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimeUnit.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).