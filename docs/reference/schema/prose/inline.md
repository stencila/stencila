---
title: Inline
description: Union type for valid inline content.
config:
  publish:
    ghost:
      type: post
      slug: inline
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Prose
---

# Members

The `Inline` type has these members:

- [`Annotation`](https://stencila.ghost.io/docs/reference/schema/annotation)
- [`AudioObject`](https://stencila.ghost.io/docs/reference/schema/audio-object)
- [`Button`](https://stencila.ghost.io/docs/reference/schema/button)
- [`Citation`](https://stencila.ghost.io/docs/reference/schema/citation)
- [`CitationGroup`](https://stencila.ghost.io/docs/reference/schema/citation-group)
- [`CodeExpression`](https://stencila.ghost.io/docs/reference/schema/code-expression)
- [`CodeInline`](https://stencila.ghost.io/docs/reference/schema/code-inline)
- [`Date`](https://stencila.ghost.io/docs/reference/schema/date)
- [`DateTime`](https://stencila.ghost.io/docs/reference/schema/date-time)
- [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration)
- [`Emphasis`](https://stencila.ghost.io/docs/reference/schema/emphasis)
- [`ImageObject`](https://stencila.ghost.io/docs/reference/schema/image-object)
- [`InstructionInline`](https://stencila.ghost.io/docs/reference/schema/instruction-inline)
- [`Link`](https://stencila.ghost.io/docs/reference/schema/link)
- [`MathInline`](https://stencila.ghost.io/docs/reference/schema/math-inline)
- [`MediaObject`](https://stencila.ghost.io/docs/reference/schema/media-object)
- [`Note`](https://stencila.ghost.io/docs/reference/schema/note)
- [`Parameter`](https://stencila.ghost.io/docs/reference/schema/parameter)
- [`QuoteInline`](https://stencila.ghost.io/docs/reference/schema/quote-inline)
- [`StyledInline`](https://stencila.ghost.io/docs/reference/schema/styled-inline)
- [`Strikeout`](https://stencila.ghost.io/docs/reference/schema/strikeout)
- [`Strong`](https://stencila.ghost.io/docs/reference/schema/strong)
- [`Subscript`](https://stencila.ghost.io/docs/reference/schema/subscript)
- [`SuggestionInline`](https://stencila.ghost.io/docs/reference/schema/suggestion-inline)
- [`Superscript`](https://stencila.ghost.io/docs/reference/schema/superscript)
- [`Text`](https://stencila.ghost.io/docs/reference/schema/text)
- [`Time`](https://stencila.ghost.io/docs/reference/schema/time)
- [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp)
- [`Underline`](https://stencila.ghost.io/docs/reference/schema/underline)
- [`VideoObject`](https://stencila.ghost.io/docs/reference/schema/video-object)
- [`Null`](https://stencila.ghost.io/docs/reference/schema/null)
- [`Boolean`](https://stencila.ghost.io/docs/reference/schema/boolean)
- [`Integer`](https://stencila.ghost.io/docs/reference/schema/integer)
- [`UnsignedInteger`](https://stencila.ghost.io/docs/reference/schema/unsigned-integer)
- [`Number`](https://stencila.ghost.io/docs/reference/schema/number)

# Bindings

The `Inline` type is represented in:

- [JSON-LD](https://stencila.org/Inline.jsonld)
- [JSON Schema](https://stencila.org/Inline.schema.json)
- Python type [`Inline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/inline.py)
- Rust type [`Inline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/inline.rs)
- TypeScript type [`Inline`](https://github.com/stencila/stencila/blob/main/ts/src/types/Inline.ts)

# Testing

During property-based (a.k.a generative) testing, the variants of the `Inline` type are generated using the following strategies[^1] for each complexity level. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

| Variant             | Complexity | Description                                                                                                                                                                                       | Strategy                                         |
| ------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| `Annotation`        | Min+       | Do not generate `Annotation` nodes in inline content.                                                                                                                                             | -                                                |
| `AudioObject`       | Min+       | Do not generate `AudioObject` nodes in inline content.                                                                                                                                            | -                                                |
|                     | High+      | Generate `AudioObject` nodes in inline content.                                                                                                                                                   | Default for level                                |
| `Button`            | Min+       | Do not generate `Button` nodes in inline content.                                                                                                                                                 | -                                                |
| `Citation`          | Min+       | Do not generate `Citation` nodes in inline content.                                                                                                                                               | -                                                |
| `CitationGroup`     | Min+       | Do not generate `CitationGroup` nodes in inline content.                                                                                                                                          | -                                                |
| `CodeExpression`    | Min+       | Do not generate `CodeExpression` nodes in inline content.                                                                                                                                         | -                                                |
|                     | Low+       | Generate `CodeExpression` nodes in inline content.                                                                                                                                                | Default for level                                |
| `Date`              | Min+       | Do not generate `Date` nodes in inline content.                                                                                                                                                   | -                                                |
|                     | High+      | Generate `Date` nodes in inline content.                                                                                                                                                          | Default for level                                |
| `DateTime`          | Min+       | Do not generate `DateTime` nodes in inline content.                                                                                                                                               | -                                                |
|                     | High+      | Generate `DateTime` nodes in inline content.                                                                                                                                                      | Default for level                                |
| `Duration`          | Min+       | Do not generate `Duration` nodes in inline content.                                                                                                                                               | -                                                |
|                     | High+      | Generate `Duration` nodes in inline content.                                                                                                                                                      | Default for level                                |
| `InstructionInline` | Min+       | Do not generate `InstructionInline` nodes in inline content.                                                                                                                                      | -                                                |
|                     | Max        | Generate `InstructionInline` nodes in inline content.                                                                                                                                             | Default for level                                |
| `MediaObject`       | Min+       | Do not generate `MediaObject` nodes in inline content.                                                                                                                                            | -                                                |
| `Note`              | Min+       | Do not generate `Note` nodes in inline content.                                                                                                                                                   | -                                                |
|                     | Low+       | Generate `Note` nodes in inline content.                                                                                                                                                          | Default for level                                |
| `Parameter`         | Min+       | Do not generate `Parameter` nodes in inline content.                                                                                                                                              | -                                                |
|                     | Low+       | Generate `Parameter` nodes in inline content.                                                                                                                                                     | Default for level                                |
| `SuggestionInline`  | Min+       | Do not generate `SuggestionInline` nodes in inline content.                                                                                                                                       | -                                                |
| `Time`              | Min+       | Do not generate `Time` nodes in inline content.                                                                                                                                                   | -                                                |
|                     | High+      | Generate `Time` nodes in inline content.                                                                                                                                                          | Default for level                                |
| `Timestamp`         | Min+       | Do not generate `Timestamp` nodes in inline content.                                                                                                                                              | -                                                |
|                     | High+      | Generate `Timestamp` nodes in inline content.                                                                                                                                                     | Default for level                                |
| `VideoObject`       | Min+       | Do not generate `VideoObject` nodes in inline content.                                                                                                                                            | -                                                |
|                     | High+      | Generate `VideoObject` nodes in inline content.                                                                                                                                                   | Default for level                                |
| `Null`              | Min+       | Do not generate `Null` nodes in inline content.                                                                                                                                                   | -                                                |
|                     | Max        | Generate a null value.                                                                                                                                                                            | `Inline::Null(Null)`                             |
| `Boolean`           | Min+       | Do not generate `Boolean` nodes in inline content.                                                                                                                                                | -                                                |
|                     | Max        | Generate an arbitrary boolean value.                                                                                                                                                              | `Boolean::arbitrary().prop_map(Inline::Boolean)` |
| `Integer`           | Min+       | Do not generate `Integer` nodes in inline content.                                                                                                                                                | -                                                |
|                     | Max        | Generate an arbitrary integer value.                                                                                                                                                              | `Integer::arbitrary().prop_map(Inline::Integer)` |
| `UnsignedInteger`   | Min+       | Do not generate `UnsignedInteger` nodes in inline content (since roundtrip<br><br>conversion can not differentiate it from an `Integer`).                                                         | -                                                |
| `Number`            | Min+       | Do not generate `Number` nodes in inline content.                                                                                                                                                 | -                                                |
|                     | Max        | Generate a fixed number. Used at all levels because even with JSON (and other data serialization formats)<br><br>round trip conversions can fail in the last significant digit of random numbers. | `Inline::Number(1.23)`                           |

# Source

This documentation was generated from [`Inline.yaml`](https://github.com/stencila/stencila/blob/main/schema/Inline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.
