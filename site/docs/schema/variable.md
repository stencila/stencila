---
title: Variable
description: A named value.
---

This is a type used in Stencila Schema for named values in executable and structured
content.

It exists to pair a symbolic name with a value within the document model,
supporting computation, templating, and data exchange workflows. Representing
variables structurally makes them easier to inspect, validate, and serialize
than ad hoc key-value text.

Key properties include the variable `name` and `value`, along with any
associated metadata or hints.


This type is marked as unstable and is subject to change.

# Analogues

The following external types, elements, or nodes are similar to a `Variable`:

- [programming language variable binding](https://en.wikipedia.org/wiki/Variable_(computer_science)): Close conceptual analogue for a named value, though Stencila exposes variables as document-model nodes with type hints and serialization semantics.

# Properties

The `Variable` type has these properties:

| Name                  | Description                                                                           | Type                    | Inherited from          |
| --------------------- | ------------------------------------------------------------------------------------- | ----------------------- | ----------------------- |
| `name`                | The name of the variable.                                                             | [`String`](./string.md) | -                       |
| `programmingLanguage` | The programming language that the variable is defined in e.g. Python, JSON.           | [`String`](./string.md) | -                       |
| `nativeType`          | The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame` | [`String`](./string.md) | -                       |
| `nodeType`            | The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`.        | [`String`](./string.md) | -                       |
| `value`               | The value of the variable.                                                            | [`Node`](./node.md)     | -                       |
| `hint`                | A hint to the value and/or structure of the variable.                                 | [`Hint`](./hint.md)     | -                       |
| `nativeHint`          | A textual hint to the value and/or structure of the variable.                         | [`String`](./string.md) | -                       |
| `id`                  | The identifier for this item.                                                         | [`String`](./string.md) | [`Entity`](./entity.md) |

# Related

The `Variable` type is related to these types:

- Parents: [`Entity`](./entity.md)
- Children: none

# Bindings

The `Variable` type is represented in:

- [JSON-LD](https://stencila.org/Variable.jsonld)
- [JSON Schema](https://stencila.org/Variable.schema.json)
- Python class [`Variable`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust struct [`Variable`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/variable.rs)
- TypeScript class [`Variable`](https://github.com/stencila/stencila/blob/main/ts/src/types/Variable.ts)

***

This documentation was generated from [`Variable.yaml`](https://github.com/stencila/stencila/blob/main/schema/Variable.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
