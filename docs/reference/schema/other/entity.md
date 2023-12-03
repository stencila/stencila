# Entity

**Abstract base type for compound (ie. non-atomic) nodes.**

This type exists mainly to have a more simple base class than schema.org's `Thing`.
This schema includes special properties that are analogous to JSON-LDs `@type` and `@id`.


**`@id`**: `stencila:Entity`

## Properties

The `Entity` type has these properties:

| Name | Aliases | `@id`                                | Type                                                                                            | Description                   | Inherited from |
| ---- | ------- | ------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------- | -------------- |
| `id` | -       | [`schema:id`](https://schema.org/id) | [`String`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md) | The identifier for this item. | -              |

## Related

The `Entity` type is related to these types:

- Parents: none
- Children: [`Admonition`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md), [`ArrayValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array-validator.md), [`BooleanValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean-validator.md), [`Cite`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md), [`CiteGroup`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite-group.md), [`CodeLocation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/code-location.md), [`CodeStatic`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-static.md), [`CompilationDigest`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation-digest.md), [`CompilationError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation-error.md), [`ConstantValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/constant-validator.md), [`Date`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md), [`DateTime`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-time.md), [`DateTimeValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-time-validator.md), [`DateValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date-validator.md), [`Duration`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md), [`DurationValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration-validator.md), [`EnumValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/enum-validator.md), [`Executable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md), [`ExecutionDependant`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependant.md), [`ExecutionDependency`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-dependency.md), [`ExecutionError`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution-error.md), [`ExecutionTag`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-tag.md), [`Function`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/function.md), [`Heading`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md), [`Instruct`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruct.md), [`Link`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md), [`List`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md), [`Mark`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/mark.md), [`Math`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math.md), [`ModifyOperation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify-operation.md), [`Note`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/note.md), [`NumberValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number-validator.md), [`Paragraph`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md), [`QuoteBlock`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote-block.md), [`Section`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md), [`StringOperation`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/string-operation.md), [`StringPatch`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/string-patch.md), [`StringValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string-validator.md), [`Styled`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled.md), [`Suggestion`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion.md), [`TableCell`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table-cell.md), [`TableRow`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table-row.md), [`Text`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md), [`ThematicBreak`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/thematic-break.md), [`Thing`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md), [`Time`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md), [`TimeValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time-validator.md), [`Timestamp`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md), [`TimestampValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp-validator.md), [`TupleValidator`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/tuple-validator.md), [`Variable`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md)

## Bindings

The `Entity` type is represented in these bindings:

- [JSON-LD](https://stencila.org/Entity.jsonld)
- [JSON Schema](https://stencila.org/Entity.schema.json)
- Python class [`Entity`](https://github.com/stencila/stencila/blob/main/python/python/stencila/types/entity.py)
- Rust struct [`Entity`](https://github.com/stencila/stencila/blob/main/rust/schema/src/types/entity.rs)
- TypeScript class [`Entity`](https://github.com/stencila/stencila/blob/main/ts/src/types/Entity.ts)

## Source

This documentation was generated from [`Entity.yaml`](https://github.com/stencila/stencila/blob/main/schema/Entity.yaml) by [`docs.rs`](https://github.com/stencila/stencila/blob/main/rust/schema-gen/src/docs.rs).