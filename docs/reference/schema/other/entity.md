---
title:
- type: Text
  value: Entity
---

# Entity

**Abstract base type for compound (ie. non-atomic) nodes.**

This type exists mainly to have a more simple base class than schema.org's `Thing`.
This schema includes special properties that are analogous to JSON-LDs `@type` and `@id`.


**`@id`**: `stencila:Entity`

## Properties

The `Entity` type has these properties:

| Name | `@id`                                | Type                                                               | Description                  | Inherited from                                                      |
| ---- | ------------------------------------ | ------------------------------------------------------------------ | ---------------------------- | ------------------------------------------------------------------- |
| id   | [`schema:id`](https://schema.org/id) | [`String`](https://stencila.dev/docs/reference/schema/data/string) | The identifier for this item | [`Entity`](https://stencila.dev/docs/reference/schema/other/entity) |

## Related

The `Entity` type is related to these types:

- Parents: none
- Children: [`ArrayValidator`](https://stencila.dev/docs/reference/schema/data/array-validator), [`BooleanValidator`](https://stencila.dev/docs/reference/schema/data/boolean-validator), [`Cite`](https://stencila.dev/docs/reference/schema/prose/cite), [`CiteGroup`](https://stencila.dev/docs/reference/schema/prose/cite-group), [`CodeError`](https://stencila.dev/docs/reference/schema/code/code-error), [`CodeStatic`](https://stencila.dev/docs/reference/schema/code/code-static), [`ConstantValidator`](https://stencila.dev/docs/reference/schema/data/constant-validator), [`Date`](https://stencila.dev/docs/reference/schema/data/date), [`DateTime`](https://stencila.dev/docs/reference/schema/data/date-time), [`DateTimeValidator`](https://stencila.dev/docs/reference/schema/data/date-time-validator), [`DateValidator`](https://stencila.dev/docs/reference/schema/data/date-validator), [`Duration`](https://stencila.dev/docs/reference/schema/data/duration), [`DurationValidator`](https://stencila.dev/docs/reference/schema/data/duration-validator), [`EnumValidator`](https://stencila.dev/docs/reference/schema/data/enum-validator), [`Executable`](https://stencila.dev/docs/reference/schema/flow/executable), [`ExecutionDependant`](https://stencila.dev/docs/reference/schema/flow/execution-dependant), [`ExecutionDependency`](https://stencila.dev/docs/reference/schema/flow/execution-dependency), [`ExecutionDigest`](https://stencila.dev/docs/reference/schema/flow/execution-digest), [`ExecutionTag`](https://stencila.dev/docs/reference/schema/flow/execution-tag), [`Function`](https://stencila.dev/docs/reference/schema/flow/function), [`Heading`](https://stencila.dev/docs/reference/schema/prose/heading), [`Link`](https://stencila.dev/docs/reference/schema/prose/link), [`List`](https://stencila.dev/docs/reference/schema/prose/list), [`Mark`](https://stencila.dev/docs/reference/schema/prose/mark), [`Math`](https://stencila.dev/docs/reference/schema/math/math), [`Note`](https://stencila.dev/docs/reference/schema/prose/note), [`NumberValidator`](https://stencila.dev/docs/reference/schema/data/number-validator), [`Paragraph`](https://stencila.dev/docs/reference/schema/prose/paragraph), [`QuoteBlock`](https://stencila.dev/docs/reference/schema/prose/quote-block), [`StringValidator`](https://stencila.dev/docs/reference/schema/data/string-validator), [`Styled`](https://stencila.dev/docs/reference/schema/style/styled), [`TableCell`](https://stencila.dev/docs/reference/schema/works/table-cell), [`TableRow`](https://stencila.dev/docs/reference/schema/works/table-row), [`Text`](https://stencila.dev/docs/reference/schema/prose/text), [`ThematicBreak`](https://stencila.dev/docs/reference/schema/prose/thematic-break), [`Thing`](https://stencila.dev/docs/reference/schema/other/thing), [`Time`](https://stencila.dev/docs/reference/schema/data/time), [`TimeValidator`](https://stencila.dev/docs/reference/schema/data/time-validator), [`Timestamp`](https://stencila.dev/docs/reference/schema/data/timestamp), [`TimestampValidator`](https://stencila.dev/docs/reference/schema/data/timestamp-validator), [`TupleValidator`](https://stencila.dev/docs/reference/schema/data/tuple-validator), [`Variable`](https://stencila.dev/docs/reference/schema/flow/variable)

## Bindings

The `Entity` type is represented in these bindings:

- [JSON-LD](https://stencila.dev/Entity.jsonld)
- [JSON Schema](https://stencila.dev/Entity.schema.json)
- Python class [`Entity`](https://github.com/stencila/stencila/blob/main/python/stencila/types/entity.py)
- Rust struct [`Entity`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/entity.rs)
- TypeScript class [`Entity`](https://github.com/stencila/stencila/blob/main/typescript/src/types/Entity.ts)

## Source

This documentation was generated from [`Entity.yaml`](https://github.com/stencila/stencila/blob/main/schema/Entity.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).