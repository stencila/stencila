# List Order

**Indicates how a `List` is ordered.**

**`@id`**: [`schema:ItemListOrderType`](https://schema.org/ItemListOrderType)

## Members

The `ListOrder` type has these members:

- `Ascending`
- `Descending`
- `Unordered`

## Bindings

The `ListOrder` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/ListOrder.jsonld)
- [JSON Schema](https://stencila.dev/ListOrder.schema.json)
- Python type [`ListOrder`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/list_order.py)
- Rust type [`ListOrder`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/list_order.rs)
- TypeScript type [`ListOrder`](https://github.com/stencila/stencila/blob/main/typescript/src/types/ListOrder.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `ListOrder` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

|         |            |             |          |
| ------- | ---------- | ----------- | -------- |
| Variant | Complexity | Description | Strategy |

## Source

This documentation was generated from [`ListOrder.yaml`](https://github.com/stencila/stencila/blob/main/schema/ListOrder.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).