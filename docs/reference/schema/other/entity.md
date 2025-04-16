---
title: Entity
description: Abstract base type for compound (ie. non-atomic) nodes.
config:
  publish:
    ghost:
      type: post
      slug: entity
      state: publish
      tags:
      - '#doc'
      - '#schema'
      - Other
---

This type exists mainly to have a more simple base class than schema.org's `Thing`.
This schema includes special properties that are analogous to JSON-LDs `@type` and `@id`.


# Properties

The `Entity` type has these properties:

| Name | Description                   | Type                                                               | Inherited from | `JSON-LD @id`                        | Aliases |
| ---- | ----------------------------- | ------------------------------------------------------------------ | -------------- | ------------------------------------ | ------- |
| `id` | The identifier for this item. | [`String`](https://stencila.ghost.io/docs/reference/schema/string) | -              | [`schema:id`](https://schema.org/id) | -       |

# Related

The `Entity` type is related to these types:

- Parents: None
- Children: [`Admonition`](https://stencila.ghost.io/docs/reference/schema/admonition), [`ArrayHint`](https://stencila.ghost.io/docs/reference/schema/array-hint), [`ArrayValidator`](https://stencila.ghost.io/docs/reference/schema/array-validator), [`BooleanValidator`](https://stencila.ghost.io/docs/reference/schema/boolean-validator), [`ChatMessageGroup`](https://stencila.ghost.io/docs/reference/schema/chat-message-group), [`Citation`](https://stencila.ghost.io/docs/reference/schema/citation), [`CitationGroup`](https://stencila.ghost.io/docs/reference/schema/citation-group), [`CodeLocation`](https://stencila.ghost.io/docs/reference/schema/code-location), [`CodeStatic`](https://stencila.ghost.io/docs/reference/schema/code-static), [`CompilationDigest`](https://stencila.ghost.io/docs/reference/schema/compilation-digest), [`CompilationMessage`](https://stencila.ghost.io/docs/reference/schema/compilation-message), [`ConstantValidator`](https://stencila.ghost.io/docs/reference/schema/constant-validator), [`DatatableColumn`](https://stencila.ghost.io/docs/reference/schema/datatable-column), [`DatatableColumnHint`](https://stencila.ghost.io/docs/reference/schema/datatable-column-hint), [`DatatableHint`](https://stencila.ghost.io/docs/reference/schema/datatable-hint), [`Date`](https://stencila.ghost.io/docs/reference/schema/date), [`DateTime`](https://stencila.ghost.io/docs/reference/schema/date-time), [`DateTimeValidator`](https://stencila.ghost.io/docs/reference/schema/date-time-validator), [`DateValidator`](https://stencila.ghost.io/docs/reference/schema/date-validator), [`Directory`](https://stencila.ghost.io/docs/reference/schema/directory), [`Duration`](https://stencila.ghost.io/docs/reference/schema/duration), [`DurationValidator`](https://stencila.ghost.io/docs/reference/schema/duration-validator), [`EnumValidator`](https://stencila.ghost.io/docs/reference/schema/enum-validator), [`Excerpt`](https://stencila.ghost.io/docs/reference/schema/excerpt), [`Executable`](https://stencila.ghost.io/docs/reference/schema/executable), [`ExecutionDependant`](https://stencila.ghost.io/docs/reference/schema/execution-dependant), [`ExecutionDependency`](https://stencila.ghost.io/docs/reference/schema/execution-dependency), [`ExecutionMessage`](https://stencila.ghost.io/docs/reference/schema/execution-message), [`ExecutionTag`](https://stencila.ghost.io/docs/reference/schema/execution-tag), [`File`](https://stencila.ghost.io/docs/reference/schema/file), [`Function`](https://stencila.ghost.io/docs/reference/schema/function), [`Heading`](https://stencila.ghost.io/docs/reference/schema/heading), [`InstructionMessage`](https://stencila.ghost.io/docs/reference/schema/instruction-message), [`Link`](https://stencila.ghost.io/docs/reference/schema/link), [`List`](https://stencila.ghost.io/docs/reference/schema/list), [`Mark`](https://stencila.ghost.io/docs/reference/schema/mark), [`Math`](https://stencila.ghost.io/docs/reference/schema/math), [`ModelParameters`](https://stencila.ghost.io/docs/reference/schema/model-parameters), [`Note`](https://stencila.ghost.io/docs/reference/schema/note), [`NumberValidator`](https://stencila.ghost.io/docs/reference/schema/number-validator), [`ObjectHint`](https://stencila.ghost.io/docs/reference/schema/object-hint), [`Paragraph`](https://stencila.ghost.io/docs/reference/schema/paragraph), [`ProvenanceCount`](https://stencila.ghost.io/docs/reference/schema/provenance-count), [`QuoteBlock`](https://stencila.ghost.io/docs/reference/schema/quote-block), [`RawBlock`](https://stencila.ghost.io/docs/reference/schema/raw-block), [`Reference`](https://stencila.ghost.io/docs/reference/schema/reference), [`Role`](https://stencila.ghost.io/docs/reference/schema/role), [`Section`](https://stencila.ghost.io/docs/reference/schema/section), [`StringHint`](https://stencila.ghost.io/docs/reference/schema/string-hint), [`StringValidator`](https://stencila.ghost.io/docs/reference/schema/string-validator), [`Styled`](https://stencila.ghost.io/docs/reference/schema/styled), [`Suggestion`](https://stencila.ghost.io/docs/reference/schema/suggestion), [`TableCell`](https://stencila.ghost.io/docs/reference/schema/table-cell), [`TableRow`](https://stencila.ghost.io/docs/reference/schema/table-row), [`Text`](https://stencila.ghost.io/docs/reference/schema/text), [`ThematicBreak`](https://stencila.ghost.io/docs/reference/schema/thematic-break), [`Thing`](https://stencila.ghost.io/docs/reference/schema/thing), [`Time`](https://stencila.ghost.io/docs/reference/schema/time), [`TimeValidator`](https://stencila.ghost.io/docs/reference/schema/time-validator), [`Timestamp`](https://stencila.ghost.io/docs/reference/schema/timestamp), [`TimestampValidator`](https://stencila.ghost.io/docs/reference/schema/timestamp-validator), [`TupleValidator`](https://stencila.ghost.io/docs/reference/schema/tuple-validator), [`Unknown`](https://stencila.ghost.io/docs/reference/schema/unknown), [`Variable`](https://stencila.ghost.io/docs/reference/schema/variable), [`Walkthrough`](https://stencila.ghost.io/docs/reference/schema/walkthrough), [`WalkthroughStep`](https://stencila.ghost.io/docs/reference/schema/walkthrough-step)

# Bindings

The `Entity` type is represented in:

- [JSON-LD](https://stencila.org/Entity.jsonld)
- [JSON Schema](https://stencila.org/Entity.schema.json)
- Python class [`Entity`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/entity.py)
- Rust struct [`Entity`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/entity.rs)
- TypeScript class [`Entity`](https://github.com/stencila/stencila/blob/main/ts/src/types/Entity.ts)

# Source

This documentation was generated from [`Entity.yaml`](https://github.com/stencila/stencila/blob/main/schema/Entity.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
