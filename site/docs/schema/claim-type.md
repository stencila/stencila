---
title: Claim Type
description: A category of claim.
---

This is an enumeration used in Stencila Schema for classifying [`Claim`](./claim.md)
nodes.

It exists so claim-oriented workflows can distinguish different kinds of
reviewable statements without relying on free-text labels. The enumeration
supports consistent downstream behavior in interfaces, validation, and
publishing.

See [`Claim.claimType`](./claim.md#claimtype) for the property that uses this
enumeration.


# Members

The `ClaimType` type has these members:

| Member        | Description |
| ------------- | ----------- |
| `Statement`   | -           |
| `Theorem`     | -           |
| `Lemma`       | -           |
| `Proof`       | -           |
| `Postulate`   | -           |
| `Hypothesis`  | -           |
| `Proposition` | -           |
| `Corollary`   | -           |

# Bindings

The `ClaimType` type is represented in:

- [JSON-LD](https://stencila.org/ClaimType.jsonld)
- [JSON Schema](https://stencila.org/ClaimType.schema.json)
- Python type [`ClaimType`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`ClaimType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/claim_type.rs)
- TypeScript type [`ClaimType`](https://github.com/stencila/stencila/blob/main/ts/src/types/ClaimType.ts)

***

This documentation was generated from [`ClaimType.yaml`](https://github.com/stencila/stencila/blob/main/schema/ClaimType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
