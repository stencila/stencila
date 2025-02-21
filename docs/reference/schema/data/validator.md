---
title: Validator
description: Union type for validators.
config:
  publish:
    ghost:
      type: page
      slug: validator
      state: publish
      tags:
      - '#schema'
      - '#doc'
      - Data
---

## Members

The `Validator` type has these members:

- [`ArrayValidator`](https://stencila.ghost.io/docs/reference/schema/array-validator)
- [`BooleanValidator`](https://stencila.ghost.io/docs/reference/schema/boolean-validator)
- [`ConstantValidator`](https://stencila.ghost.io/docs/reference/schema/constant-validator)
- [`DateTimeValidator`](https://stencila.ghost.io/docs/reference/schema/date-time-validator)
- [`DateValidator`](https://stencila.ghost.io/docs/reference/schema/date-validator)
- [`DurationValidator`](https://stencila.ghost.io/docs/reference/schema/duration-validator)
- [`EnumValidator`](https://stencila.ghost.io/docs/reference/schema/enum-validator)
- [`IntegerValidator`](https://stencila.ghost.io/docs/reference/schema/integer-validator)
- [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator)
- [`StringValidator`](https://stencila.ghost.io/docs/reference/schema/string-validator)
- [`TimeValidator`](https://stencila.ghost.io/docs/reference/schema/time-validator)
- [`TimestampValidator`](https://stencila.ghost.io/docs/reference/schema/timestamp-validator)
- [`TupleValidator`](https://stencila.ghost.io/docs/reference/schema/tuple-validator)

## Bindings

The `Validator` type is represented in:

- [JSON-LD](https://stencila.org/Validator.jsonld)
- [JSON Schema](https://stencila.org/Validator.schema.json)
- Python type [`Validator`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/validator.py)
- Rust type [`Validator`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/validator.rs)
- TypeScript type [`Validator`](https://github.com/stencila/stencila/blob/main/ts/src/types/Validator.ts)

## Source

This documentation was generated from [`Validator.yaml`](https://github.com/stencila/stencila/blob/main/schema/Validator.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
