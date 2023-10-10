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
- [`CodeFragment`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-fragment.md)
- [`Date`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)
- [`DateTime`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-time.md)
- [`Delete`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/delete.md)
- [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)
- [`Emphasis`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/emphasis.md)
- [`ImageObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md)
- [`Insert`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/insert.md)
- [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)
- [`MathFragment`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math-fragment.md)
- [`MediaObject`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/media-object.md)
- [`Note`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/note.md)
- [`Parameter`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/parameter.md)
- [`Quote`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote.md)
- [`Span`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/span.md)
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
- [`Number`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md)
- [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)

## Bindings

The `Inline` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Inline.jsonld)
- [JSON Schema](https://stencila.dev/Inline.schema.json)
- Python type [`Inline`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/inline.py)
- Rust type [`Inline`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/inline.rs)
- TypeScript type [`Inline`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Inline.ts)

## Testing

During property-based (a.k.a generative) testing, the variants of the `Inline` type are generated using the following strategies for each complexity level (see the [`proptest` book](https://proptest-rs.github.io/proptest/) for an explanation of the Rust strategy expressions). Any variant not shown is generated using the default strategy for the corresponding type and complexity level.

| Variant          | Complexity | Description                                                                                                                                                                                 | Strategy                                         |
| ---------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| `Button`         | Min+       | Do not generate `Button` nodes in inline content.                                                                                                                                           | -                                                |
| `Cite`           | Min+       | Do not generate `Cite` nodes in inline content.                                                                                                                                             | -                                                |
| `CiteGroup`      | Min+       | Do not generate `CiteGroup` nodes in inline content.                                                                                                                                        | -                                                |
| `CodeExpression` | Min+       | Do not generate `CodeExpression` nodes in inline content.                                                                                                                                   | -                                                |
|                  | Low+       | Generate `CodeExpression` nodes in inline content.                                                                                                                                          | Default for level                                |
| `Date`           | Min+       | Do not generate `Date` nodes in inline content.                                                                                                                                             | -                                                |
|                  | Low+       | Generate `Date` nodes in inline content.                                                                                                                                                    | Default for level                                |
| `DateTime`       | Min+       | Do not generate `DateTime` nodes in inline content.                                                                                                                                         | -                                                |
|                  | Low+       | Generate `DateTime` nodes in inline content.                                                                                                                                                | Default for level                                |
| `Delete`         | Min+       | Do not generate `Delete` nodes in inline content.                                                                                                                                           | -                                                |
|                  | Low+       | Generate `Delete` nodes in inline content.                                                                                                                                                  | Default for level                                |
| `Insert`         | Min+       | Do not generate `Insert` nodes in inline content.                                                                                                                                           | -                                                |
|                  | Low+       | Generate `Insert` nodes in inline content.                                                                                                                                                  | Default for level                                |
| `MediaObject`    | Min+       | Do not generate `MediaObject` nodes in inline content.                                                                                                                                      | -                                                |
| `Note`           | Min+       | Do not generate `Note` nodes in inline content.                                                                                                                                             | -                                                |
|                  | Low+       | Generate `Note` nodes in inline content.                                                                                                                                                    | Default for level                                |
| `Parameter`      | Min+       | Do not generate `Parameter` nodes in inline content.                                                                                                                                        | -                                                |
| `Time`           | Min+       | Do not generate `Time` nodes in inline content.                                                                                                                                             | -                                                |
|                  | Low+       | Generate `Time` nodes in inline content.                                                                                                                                                    | Default for level                                |
| `Timestamp`      | Min+       | Do not generate `Timestamp` nodes in inline content.                                                                                                                                        | -                                                |
|                  | Low+       | Generate `Timestamp` nodes in inline content.                                                                                                                                               | Default for level                                |
| `Null`           | Min+       | Do not generate `Null` nodes in inline content.                                                                                                                                             | -                                                |
|                  | High+      | Generate a null value.                                                                                                                                                                      | `Inline::Null(Null)`                             |
| `Boolean`        | Min+       | Do not generate `Boolean` nodes in inline content.                                                                                                                                          | -                                                |
|                  | High+      | Generate an arbitrary boolean value.                                                                                                                                                        | `Boolean::arbitrary().prop_map(Inline::Boolean)` |
| `Integer`        | Min+       | Do not generate `Integer` nodes in inline content.                                                                                                                                          | -                                                |
|                  | High+      | Generate an arbitrary integer value.                                                                                                                                                        | `Integer::arbitrary().prop_map(Inline::Integer)` |
| `Number`         | Min+       | Do not generate `Number` nodes in inline content.                                                                                                                                           | -                                                |
|                  | High+      | Generate a fixed number. Used at all levels because even with JSON (and other data serialization formats) round trip conversions can fail in the last significant digit of random numbers.  | `Inline::Number(1.23)`                           |
| `String`         | Min+       | Do not generate `String` nodes in inline content. Skipped at all levels because `Text` is preferred and `String` is likely to deprecated as an inline variant.                              | -                                                |

## Source

This documentation was generated from [`Inline.yaml`](https://github.com/stencila/stencila/blob/main/schema/Inline.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).