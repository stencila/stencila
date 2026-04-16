---
title: Hint
description: Union type for hints of the value and/or structure of data.
---

This is a union type used in Stencila Schema for data hints.

It groups together the hint node types used to provide concise summaries of
data values and structures without imposing hard validation rules. These
summaries are useful to both humans and machines, including user interfaces,
schema inference, and code generation workflows such as choosing appropriate
visualizations for large datasets.

Use this type to understand what hint nodes can appear in data-oriented parts
of the schema.


# Members

The `Hint` type has these members:

- [`ArrayHint`](./array-hint.md)
- [`DatatableHint`](./datatable-hint.md)
- [`Function`](./function.md)
- [`ObjectHint`](./object-hint.md)
- [`StringHint`](./string-hint.md)
- [`Unknown`](./unknown.md)
- [`Boolean`](./boolean.md)
- [`Integer`](./integer.md)
- [`Number`](./number.md)

# Bindings

The `Hint` type is represented in:

- [JSON-LD](https://stencila.org/Hint.jsonld)
- [JSON Schema](https://stencila.org/Hint.schema.json)
- Python type [`Hint`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Hint`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/hint.rs)
- TypeScript type [`Hint`](https://github.com/stencila/stencila/blob/main/ts/src/types/Hint.ts)

***

This documentation was generated from [`Hint.yaml`](https://github.com/stencila/stencila/blob/main/schema/Hint.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
