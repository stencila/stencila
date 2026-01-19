---
title: Entity
description: Abstract base type for compound (ie. non-atomic) nodes.
---

This type exists mainly to have a more simple base class than schema.org's `Thing`.
This schema includes special properties that are analogous to JSON-LDs `@type` and `@id`.


# Properties

The `Entity` type has these properties:

| Name | Description                   | Type                    | Inherited from | `JSON-LD @id`                        | Aliases |
| ---- | ----------------------------- | ----------------------- | -------------- | ------------------------------------ | ------- |
| `id` | The identifier for this item. | [`String`](./string.md) | -              | [`schema:id`](https://schema.org/id) | -       |

# Related

The `Entity` type is related to these types:

- Parents: None
- Children: [`Admonition`](./admonition.md), [`AppendixBreak`](./appendix-break.md), [`ArrayHint`](./array-hint.md), [`ArrayValidator`](./array-validator.md), [`BooleanValidator`](./boolean-validator.md), [`ChatMessageGroup`](./chat-message-group.md), [`Citation`](./citation.md), [`CitationGroup`](./citation-group.md), [`CodeLocation`](./code-location.md), [`CodeStatic`](./code-static.md), [`CompilationDigest`](./compilation-digest.md), [`CompilationMessage`](./compilation-message.md), [`ConstantValidator`](./constant-validator.md), [`DatatableColumn`](./datatable-column.md), [`DatatableColumnHint`](./datatable-column-hint.md), [`DatatableHint`](./datatable-hint.md), [`Date`](./date.md), [`DateTime`](./date-time.md), [`DateTimeValidator`](./date-time-validator.md), [`DateValidator`](./date-validator.md), [`Directory`](./directory.md), [`Duration`](./duration.md), [`DurationValidator`](./duration-validator.md), [`EnumValidator`](./enum-validator.md), [`Excerpt`](./excerpt.md), [`Executable`](./executable.md), [`ExecutionDependant`](./execution-dependant.md), [`ExecutionDependency`](./execution-dependency.md), [`ExecutionMessage`](./execution-message.md), [`ExecutionTag`](./execution-tag.md), [`Function`](./function.md), [`Heading`](./heading.md), [`InlinesBlock`](./inlines-block.md), [`InstructionMessage`](./instruction-message.md), [`Island`](./island.md), [`Link`](./link.md), [`List`](./list.md), [`Mark`](./mark.md), [`Math`](./math.md), [`ModelParameters`](./model-parameters.md), [`Note`](./note.md), [`NumberValidator`](./number-validator.md), [`ObjectHint`](./object-hint.md), [`Paragraph`](./paragraph.md), [`ProvenanceCount`](./provenance-count.md), [`QuoteBlock`](./quote-block.md), [`RawBlock`](./raw-block.md), [`Reference`](./reference.md), [`Role`](./role.md), [`Section`](./section.md), [`Sentence`](./sentence.md), [`StringHint`](./string-hint.md), [`StringValidator`](./string-validator.md), [`Styled`](./styled.md), [`Suggestion`](./suggestion.md), [`Supplement`](./supplement.md), [`TableCell`](./table-cell.md), [`TableRow`](./table-row.md), [`Text`](./text.md), [`ThematicBreak`](./thematic-break.md), [`Thing`](./thing.md), [`Time`](./time.md), [`TimeValidator`](./time-validator.md), [`Timestamp`](./timestamp.md), [`TimestampValidator`](./timestamp-validator.md), [`TupleValidator`](./tuple-validator.md), [`Unknown`](./unknown.md), [`Variable`](./variable.md), [`Walkthrough`](./walkthrough.md), [`WalkthroughStep`](./walkthrough-step.md)

# Bindings

The `Entity` type is represented in:

- [JSON-LD](https://stencila.org/Entity.jsonld)
- [JSON Schema](https://stencila.org/Entity.schema.json)
- Python class [`Entity`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/entity.py)
- Rust struct [`Entity`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/entity.rs)
- TypeScript class [`Entity`](https://github.com/stencila/stencila/blob/main/ts/src/types/Entity.ts)

# Source

This documentation was generated from [`Entity.yaml`](https://github.com/stencila/stencila/blob/main/schema/Entity.yaml) by [`docs_types.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs_types.rs).
