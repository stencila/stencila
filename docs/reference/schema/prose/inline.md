# Inline

**Union type for valid inline content.**

**`@id`**: `stencila:Inline`

## Members

The `Inline` type has these members:

- [`AudioObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/audio-object.md)
- [`Button`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/button.md)
- [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)
- [`CiteGroup`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite-group.md)
- [`CodeExpression`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-expression.md)
- [`CodeInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-inline.md)
- [`Date`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)
- [`DateTime`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-time.md)
- [`DeleteInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete-inline.md)
- [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)
- [`Emphasis`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/emphasis.md)
- [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)
- [`InsertInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert-inline.md)
- [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)
- [`MathInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math-inline.md)
- [`MediaObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/media-object.md)
- [`Note`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/note.md)
- [`Parameter`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/parameter.md)
- [`QuoteInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote-inline.md)
- [`StyledInline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled-inline.md)
- [`Strikeout`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strikeout.md)
- [`Strong`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strong.md)
- [`Subscript`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/subscript.md)
- [`Superscript`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/superscript.md)
- [`Text`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)
- [`Time`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)
- [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)
- [`Underline`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/underline.md)
- [`VideoObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/video-object.md)
- [`Null`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/null.md)
- [`Boolean`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)
- [`Integer`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)
- [`UnsignedInteger`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md)
- [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md)

## Bindings

The `Inline` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Inline.jsonld)
- [JSON Schema](https://stencila.org/Inline.schema.json)
- Python type [`Inline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/inline.py)
- Rust type [`Inline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/inline.rs)
- TypeScript type [`Inline`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Inline.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `Inline` type are generated using the following strategies[^1] for each complexity level. Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

| Variant           | Complexity | Description                                                                                                                                                                                       | Strategy                                         |
| ----------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| `AudioObject`     | Min+       | Do not generate `AudioObject` nodes in inline content.                                                                                                                                            | -                                                |
|                   | High+      | Generate `AudioObject` nodes in inline content.                                                                                                                                                   | Default for level                                |
| `Button`          | Min+       | Do not generate `Button` nodes in inline content.                                                                                                                                                 | -                                                |
| `Cite`            | Min+       | Do not generate `Cite` nodes in inline content.                                                                                                                                                   | -                                                |
| `CiteGroup`       | Min+       | Do not generate `CiteGroup` nodes in inline content.                                                                                                                                              | -                                                |
| `CodeExpression`  | Min+       | Do not generate `CodeExpression` nodes in inline content.                                                                                                                                         | -                                                |
|                   | Low+       | Generate `CodeExpression` nodes in inline content.                                                                                                                                                | Default for level                                |
| `Date`            | Min+       | Do not generate `Date` nodes in inline content.                                                                                                                                                   | -                                                |
|                   | High+      | Generate `Date` nodes in inline content.                                                                                                                                                          | Default for level                                |
| `DateTime`        | Min+       | Do not generate `DateTime` nodes in inline content.                                                                                                                                               | -                                                |
|                   | High+      | Generate `DateTime` nodes in inline content.                                                                                                                                                      | Default for level                                |
| `DeleteInline`    | Min+       | Do not generate `DeleteInline` nodes in inline content.                                                                                                                                           | -                                                |
|                   | Max        | Generate `DeleteInline` nodes in inline content.                                                                                                                                                  | Default for level                                |
| `Duration`        | Min+       | Do not generate `Duration` nodes in inline content.                                                                                                                                               | -                                                |
|                   | High+      | Generate `Duration` nodes in inline content.                                                                                                                                                      | Default for level                                |
| `InsertInline`    | Min+       | Do not generate `InsertInline` nodes in inline content.                                                                                                                                           | -                                                |
|                   | Max        | Generate `InsertInline` nodes in inline content.                                                                                                                                                  | Default for level                                |
| `MediaObject`     | Min+       | Do not generate `MediaObject` nodes in inline content.                                                                                                                                            | -                                                |
| `Note`            | Min+       | Do not generate `Note` nodes in inline content.                                                                                                                                                   | -                                                |
|                   | Low+       | Generate `Note` nodes in inline content.                                                                                                                                                          | Default for level                                |
| `Parameter`       | Min+       | Do not generate `Parameter` nodes in inline content.                                                                                                                                              | -                                                |
|                   | Low+       | Generate `Parameter` nodes in inline content.                                                                                                                                                     | Default for level                                |
| `Time`            | Min+       | Do not generate `Time` nodes in inline content.                                                                                                                                                   | -                                                |
|                   | High+      | Generate `Time` nodes in inline content.                                                                                                                                                          | Default for level                                |
| `Timestamp`       | Min+       | Do not generate `Timestamp` nodes in inline content.                                                                                                                                              | -                                                |
|                   | High+      | Generate `Timestamp` nodes in inline content.                                                                                                                                                     | Default for level                                |
| `VideoObject`     | Min+       | Do not generate `VideoObject` nodes in inline content.                                                                                                                                            | -                                                |
|                   | High+      | Generate `VideoObject` nodes in inline content.                                                                                                                                                   | Default for level                                |
| `Null`            | Min+       | Do not generate `Null` nodes in inline content.                                                                                                                                                   | -                                                |
|                   | Max        | Generate a null value.                                                                                                                                                                            | `Inline::Null(Null)`                             |
| `Boolean`         | Min+       | Do not generate `Boolean` nodes in inline content.                                                                                                                                                | -                                                |
|                   | Max        | Generate an arbitrary boolean value.                                                                                                                                                              | `Boolean::arbitrary().prop_map(Inline::Boolean)` |
| `Integer`         | Min+       | Do not generate `Integer` nodes in inline content.                                                                                                                                                | -                                                |
|                   | Max        | Generate an arbitrary integer value.                                                                                                                                                              | `Integer::arbitrary().prop_map(Inline::Integer)` |
| `UnsignedInteger` | Min+       | Do not generate `UnsignedInteger` nodes in inline content (since roundtrip<br><br>conversion can not differentiate it from an `Integer`).                                                         | -                                                |
| `Number`          | Min+       | Do not generate `Number` nodes in inline content.                                                                                                                                                 | -                                                |
|                   | Max        | Generate a fixed number. Used at all levels because even with JSON (and other data serialization formats)<br><br>round trip conversions can fail in the last significant digit of random numbers. | `Inline::Number(1.23)`                           |

## Source

This documentation was generated from [`Inline.yaml`](https://github.com/stencila/stencila/blob/main/schema/Inline.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).

[^1]: See the `proptest` [book](https://proptest-rs.github.io/proptest/) and the [`proptest.rs`](https://github.com/stencila/stencila/blob/main/rust/schema/src/proptests.rs) module for details.