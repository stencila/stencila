---
title: Inline
description: Union type for valid inline content.
---

# Members

The `Inline` type has these members:

- [`Annotation`](./annotation.md)
- [`AudioObject`](./audio-object.md)
- [`Button`](./button.md)
- [`Citation`](./citation.md)
- [`CitationGroup`](./citation-group.md)
- [`CodeExpression`](./code-expression.md)
- [`CodeInline`](./code-inline.md)
- [`Date`](./date.md)
- [`DateTime`](./date-time.md)
- [`Duration`](./duration.md)
- [`Emphasis`](./emphasis.md)
- [`ImageObject`](./image-object.md)
- [`InstructionInline`](./instruction-inline.md)
- [`Link`](./link.md)
- [`MathInline`](./math-inline.md)
- [`MediaObject`](./media-object.md)
- [`Note`](./note.md)
- [`Parameter`](./parameter.md)
- [`QuoteInline`](./quote-inline.md)
- [`Sentence`](./sentence.md)
- [`StyledInline`](./styled-inline.md)
- [`Strikeout`](./strikeout.md)
- [`Strong`](./strong.md)
- [`Subscript`](./subscript.md)
- [`SuggestionInline`](./suggestion-inline.md)
- [`Superscript`](./superscript.md)
- [`Text`](./text.md)
- [`Time`](./time.md)
- [`Timestamp`](./timestamp.md)
- [`Underline`](./underline.md)
- [`VideoObject`](./video-object.md)
- [`Null`](./null.md)
- [`Boolean`](./boolean.md)
- [`Integer`](./integer.md)
- [`UnsignedInteger`](./unsigned-integer.md)
- [`Number`](./number.md)

# Bindings

The `Inline` type is represented in:

- [JSON-LD](https://stencila.org/Inline.jsonld)
- [JSON Schema](https://stencila.org/Inline.schema.json)
- Python type [`Inline`](https://github.com/stencila/stencila/blob/main/python/stencila_types/src/stencila_types/types.py)
- Rust type [`Inline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/inline.rs)
- TypeScript type [`Inline`](https://github.com/stencila/stencila/blob/main/ts/src/types/Inline.ts)

# Testing

During property-based (a.k.a generative) testing, the variants of the `Inline` type are generated using the following strategies. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

::: table

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
| `Sentence`          | Min+       | Do not generate `Sentence` nodes in inline content.                                                                                                                                               | -                                                |
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

See the `proptest` [book](https://proptest-rs.github.io/proptest/) and Stencila Schema's [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details on the proptest generation strategies listed.

:::

***

This documentation was generated from [`Inline.yaml`](https://github.com/stencila/stencila/blob/main/schema/Inline.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
