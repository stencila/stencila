---
title: Time Unit
description: A unit in which time can be measured.
---

This is an enumeration used in Stencila Schema for units of time.

It exists to provide a controlled vocabulary for time-based quantities and
durations where Stencila needs symbolic units rather than free-text
abbreviations.

See properties on duration- and time-related types that reference this
enumeration.


# Analogues

The following external types, elements, or nodes are similar to a `TimeUnit`:

- [UCUM time units](https://ucum.org/): Close controlled-vocabulary analogue for units of temporal measurement, though Stencila uses a compact fixed enumeration rather than coded unit strings.

# Members

The `TimeUnit` type has these members:

| Member        | Description |
| ------------- | ----------- |
| `Year`        | -           |
| `Month`       | -           |
| `Week`        | -           |
| `Day`         | -           |
| `Hour`        | -           |
| `Minute`      | -           |
| `Second`      | -           |
| `Millisecond` | -           |
| `Microsecond` | -           |
| `Nanosecond`  | -           |
| `Picosecond`  | -           |
| `Femtosecond` | -           |
| `Attosecond`  | -           |

# Bindings

The `TimeUnit` type is represented in:

- [JSON-LD](https://stencila.org/TimeUnit.jsonld)
- [JSON Schema](https://stencila.org/TimeUnit.schema.json)
- Python type [`TimeUnit`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`TimeUnit`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/time_unit.rs)
- TypeScript type [`TimeUnit`](https://github.com/stencila/stencila/blob/main/ts/src/types/TimeUnit.ts)

***

This documentation was generated from [`TimeUnit.yaml`](https://github.com/stencila/stencila/blob/main/schema/TimeUnit.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
