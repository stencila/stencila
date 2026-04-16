---
title: Form Derive Action
description: An action for applying a derived form value.
---

This is an enumeration used in Stencila Schema describing how a derived form value
should be applied.

It exists so Stencila can represent create, update, and delete actions in a
consistent way when forms derive structured document changes.

See the form derivation properties that reference this enumeration.


# Members

The `FormDeriveAction` type has these members:

| Member           | Description |
| ---------------- | ----------- |
| `Create`         | -           |
| `Update`         | -           |
| `Delete`         | -           |
| `UpdateOrDelete` | -           |

# Bindings

The `FormDeriveAction` type is represented in:

- [JSON-LD](https://stencila.org/FormDeriveAction.jsonld)
- [JSON Schema](https://stencila.org/FormDeriveAction.schema.json)
- Python type [`FormDeriveAction`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`FormDeriveAction`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/form_derive_action.rs)
- TypeScript type [`FormDeriveAction`](https://github.com/stencila/stencila/blob/main/ts/src/types/FormDeriveAction.ts)

***

This documentation was generated from [`FormDeriveAction.yaml`](https://github.com/stencila/stencila/blob/main/schema/FormDeriveAction.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
