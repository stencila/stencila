---
title: Claim Type
description: The type of a `Claim`.
config:
  publish:
    ghost:
      type: post
      slug: claim-type
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Works
---

# Members

The `ClaimType` type has these members:

- `Statement`
- `Theorem`
- `Lemma`
- `Proof`
- `Postulate`
- `Hypothesis`
- `Proposition`
- `Corollary`

# Bindings

The `ClaimType` type is represented in:

- [JSON-LD](https://stencila.org/ClaimType.jsonld)
- [JSON Schema](https://stencila.org/ClaimType.schema.json)
- Python type [`ClaimType`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/claim_type.py)
- Rust type [`ClaimType`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/claim_type.rs)
- TypeScript type [`ClaimType`](https://github.com/stencila/stencila/blob/main/ts/src/types/ClaimType.ts)

# Source

This documentation was generated from [`ClaimType.yaml`](https://github.com/stencila/stencila/blob/main/schema/ClaimType.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
