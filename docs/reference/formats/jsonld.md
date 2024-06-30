# JSON-LD

## Introduction

[JSON-LD](https://json-ld.org/), or JSON for Linked Data, is a lightweight data interchange format designed to express linked data in a format that is both human-readable and machine-friendly. It extends JSON by providing a standard way to embed linked data within JSON documents, allowing for a network of standards-based, machine-readable, structured data on the web.

## Implementation

Stencila Schema is based on [schema.org](https://schema.org) and has a JSON-LD `@context` published at https://stencila.org/context.jsonld. When Stencila documents are exported as JSON, this context is applied. As such, the JSON documents that Stencila produces are inherently JSON-LD documents.

For example, an `Article` is exported like so:

```json
{
  "$schema": "https://stencila.org/Article.schema.json",
  "@context": "https://stencila.org/context.jsonld",
  "type": "Article",
  "content": [
    {
      "type": "Paragraph",
      "content": [
        {
          "type": "Text",
```

However, because the the schema.org is the most widely used vocabulary for JSON-LD, the `JsonLdCodec` translates terms in the Stencila context, to those in the schema.org context, and uses schema.org as the [default vocabulary](https://www.w3.org/TR/json-ld11/#default-vocabulary), with the Stencila context as an extension. This saves consumers of the JSON-LD from having to do this translation themselves.

In addition, when exporting to JSON-LD, the `@type` and `@id` [keywords](https://www.w3.org/TR/json-ld11/#syntax-tokens-and-keywords) are used instead of `type` and `id`.

For example, the above article as exported to JSON-LD as follows. Note that because the types `Article` and `Text` are part of schema.org, there is no need to prefix their name. However because schema.org does not have a `Paragraph` type or a `content` property, it is necessary to prefix those with `stencila:`.

```json
{
  "@context": {
    "@vocab": "https://schema.org/",
    "stencila": "https://stencila.org/"
  },
  "@type": "Article",
  "stencila:content": [
    {
      "@type": "stencila:Paragraph",
      "stencila:content": [
        {
          "@type": "Text",
```

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

## Codec

The codec (en**co**der/**dec**oder) for JSON-LD supports:

- decoding from a file
- decoding from a string
- encoding to a file
- encoding to a string

Support and degree of loss for node types:

| Node type                                                                                                                 | Encoding  | Decoding  | Notes |
| ------------------------------------------------------------------------------------------------------------------------- | --------- | --------- | ----- |
| **Works**                                                                                                                 |
| [Article](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/article.md)                          | 🟢 No loss | 🟢 No loss |       |
| [AudioObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/audio_object.md)                 | 🟢 No loss | 🟢 No loss |       |
| [AuthorRole](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author_role.md)                   | 🟢 No loss | 🟢 No loss |       |
| [Claim](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/claim.md)                              | 🟢 No loss | 🟢 No loss |       |
| [Collection](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/collection.md)                    | 🟢 No loss | 🟢 No loss |       |
| [Comment](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/comment.md)                          | 🟢 No loss | 🟢 No loss |       |
| [CreativeWork](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/creative_work.md)               | 🟢 No loss | 🟢 No loss |       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/directory.md)                      | 🟢 No loss | 🟢 No loss |       |
| [Figure](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/figure.md)                            | 🟢 No loss | 🟢 No loss |       |
| [File](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/file.md)                                | 🟢 No loss | 🟢 No loss |       |
| [ImageObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image_object.md)                 | 🟢 No loss | 🟢 No loss |       |
| [MediaObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/media_object.md)                 | 🟢 No loss | 🟢 No loss |       |
| [Periodical](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/periodical.md)                    | 🟢 No loss | 🟢 No loss |       |
| [PublicationIssue](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/publication_issue.md)       | 🟢 No loss | 🟢 No loss |       |
| [PublicationVolume](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/publication_volume.md)     | 🟢 No loss | 🟢 No loss |       |
| [Review](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/review.md)                            | 🟢 No loss | 🟢 No loss |       |
| [SoftwareApplication](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software_application.md) | 🟢 No loss | 🟢 No loss |       |
| [SoftwareSourceCode](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software_source_code.md)  | 🟢 No loss | 🟢 No loss |       |
| [Table](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table.md)                              | 🟢 No loss | 🟢 No loss |       |
| [TableCell](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table_cell.md)                     | 🟢 No loss | 🟢 No loss |       |
| [TableRow](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table_row.md)                       | 🟢 No loss | 🟢 No loss |       |
| [VideoObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/video_object.md)                 | 🟢 No loss | 🟢 No loss |       |
| **Prose**                                                                                                                 |
| [Admonition](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md)                    | 🟢 No loss | 🟢 No loss |       |
| [Cite](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)                                | 🟢 No loss | 🟢 No loss |       |
| [CiteGroup](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite_group.md)                     | 🟢 No loss | 🟢 No loss |       |
| [DefinedTerm](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/defined_term.md)                 | 🟢 No loss | 🟢 No loss |       |
| [Emphasis](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/emphasis.md)                        | 🟢 No loss | 🟢 No loss |       |
| [Heading](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md)                          | 🟢 No loss | 🟢 No loss |       |
| [Link](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)                                | 🟢 No loss | 🟢 No loss |       |
| [List](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md)                                | 🟢 No loss | 🟢 No loss |       |
| [ListItem](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list_item.md)                       | 🟢 No loss | 🟢 No loss |       |
| [Note](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/note.md)                                | 🟢 No loss | 🟢 No loss |       |
| [Paragraph](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md)                      | 🟢 No loss | 🟢 No loss |       |
| [QuoteBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote_block.md)                   | 🟢 No loss | 🟢 No loss |       |
| [QuoteInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote_inline.md)                 | 🟢 No loss | 🟢 No loss |       |
| [Section](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md)                          | 🟢 No loss | 🟢 No loss |       |
| [Strikeout](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strikeout.md)                      | 🟢 No loss | 🟢 No loss |       |
| [Strong](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strong.md)                            | 🟢 No loss | 🟢 No loss |       |
| [Subscript](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/subscript.md)                      | 🟢 No loss | 🟢 No loss |       |
| [Superscript](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/superscript.md)                  | 🟢 No loss | 🟢 No loss |       |
| [Text](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)                                | 🟢 No loss | 🟢 No loss |       |
| [ThematicBreak](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/thematic_break.md)             | 🟢 No loss | 🟢 No loss |       |
| [Underline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/underline.md)                      | 🟢 No loss | 🟢 No loss |       |
| **Math**                                                                                                                  |
| [MathBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math_block.md)                      | 🟢 No loss | 🟢 No loss |       |
| [MathInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math_inline.md)                    | 🟢 No loss | 🟢 No loss |       |
| **Code**                                                                                                                  |
| [CodeBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_block.md)                      | 🟢 No loss | 🟢 No loss |       |
| [CodeChunk](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_chunk.md)                      | 🟢 No loss | 🟢 No loss |       |
| [CodeExpression](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_expression.md)            | 🟢 No loss | 🟢 No loss |       |
| [CodeInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_inline.md)                    | 🟢 No loss | 🟢 No loss |       |
| [CompilationMessage](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation_message.md)    | 🟢 No loss | 🟢 No loss |       |
| [ExecutionMessage](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution_message.md)        | 🟢 No loss | 🟢 No loss |       |
| **Data**                                                                                                                  |
| [Array](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array.md)                               | 🟢 No loss | 🟢 No loss |       |
| [ArrayHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array_hint.md)                      | 🟢 No loss | 🟢 No loss |       |
| [ArrayValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array_validator.md)            | 🟢 No loss | 🟢 No loss |       |
| [Boolean](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)                           | 🟢 No loss | 🟢 No loss |       |
| [BooleanValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean_validator.md)        | 🟢 No loss | 🟢 No loss |       |
| [ConstantValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/constant_validator.md)      | 🟢 No loss | 🟢 No loss |       |
| [Cord](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                                 | 🟢 No loss | 🟢 No loss |       |
| [Datatable](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable.md)                       | 🟢 No loss | 🟢 No loss |       |
| [DatatableColumn](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable_column.md)          | 🟢 No loss | 🟢 No loss |       |
| [DatatableColumnHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable_column_hint.md) | 🟢 No loss | 🟢 No loss |       |
| [DatatableHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable_hint.md)              | 🟢 No loss | 🟢 No loss |       |
| [Date](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)                                 | 🟢 No loss | 🟢 No loss |       |
| [DateTime](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_time.md)                        | 🟢 No loss | 🟢 No loss |       |
| [DateTimeValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_time_validator.md)     | 🟢 No loss | 🟢 No loss |       |
| [DateValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_validator.md)              | 🟢 No loss | 🟢 No loss |       |
| [Duration](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                         | 🟢 No loss | 🟢 No loss |       |
| [DurationValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration_validator.md)      | 🟢 No loss | 🟢 No loss |       |
| [EnumValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/enum_validator.md)              | 🟢 No loss | 🟢 No loss |       |
| [Integer](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                           | 🟢 No loss | 🟢 No loss |       |
| [IntegerValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer_validator.md)        | 🟢 No loss | 🟢 No loss |       |
| [Null](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/null.md)                                 | 🟢 No loss | 🟢 No loss |       |
| [Number](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md)                             | 🟢 No loss | 🟢 No loss |       |
| [NumberValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number_validator.md)          | 🟢 No loss | 🟢 No loss |       |
| [Object](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/object.md)                             | 🟢 No loss | 🟢 No loss |       |
| [ObjectHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/object_hint.md)                    | 🟢 No loss | 🟢 No loss |       |
| [String](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | 🟢 No loss | 🟢 No loss |       |
| [StringHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string_hint.md)                    | 🟢 No loss | 🟢 No loss |       |
| [StringValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string_validator.md)          | 🟢 No loss | 🟢 No loss |       |
| [Time](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)                                 | 🟢 No loss | 🟢 No loss |       |
| [TimeValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time_validator.md)              | 🟢 No loss | 🟢 No loss |       |
| [Timestamp](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                       | 🟢 No loss | 🟢 No loss |       |
| [TimestampValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp_validator.md)    | 🟢 No loss | 🟢 No loss |       |
| [TupleValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/tuple_validator.md)            | 🟢 No loss | 🟢 No loss |       |
| [Unknown](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unknown.md)                           | 🟢 No loss | 🟢 No loss |       |
| [UnsignedInteger](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned_integer.md)          | 🟢 No loss | 🟢 No loss |       |
| **Flow**                                                                                                                  |
| [Button](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/button.md)                             | 🟢 No loss | 🟢 No loss |       |
| [CallArgument](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call_argument.md)                | 🟢 No loss | 🟢 No loss |       |
| [CallBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call_block.md)                      | 🟢 No loss | 🟢 No loss |       |
| [CodeLocation](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/code_location.md)                | 🟢 No loss | 🟢 No loss |       |
| [CompilationDigest](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation_digest.md)      | 🟢 No loss | 🟢 No loss |       |
| [ExecutionDependant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_dependant.md)    | 🟢 No loss | 🟢 No loss |       |
| [ExecutionDependency](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_dependency.md)  | 🟢 No loss | 🟢 No loss |       |
| [ExecutionTag](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_tag.md)                | 🟢 No loss | 🟢 No loss |       |
| [ForBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for_block.md)                        | 🟢 No loss | 🟢 No loss |       |
| [Form](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)                                 | 🟢 No loss | 🟢 No loss |       |
| [Function](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/function.md)                         | 🟢 No loss | 🟢 No loss |       |
| [IfBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if_block.md)                          | 🟢 No loss | 🟢 No loss |       |
| [IfBlockClause](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if_block_clause.md)             | 🟢 No loss | 🟢 No loss |       |
| [IncludeBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include_block.md)                | 🟢 No loss | 🟢 No loss |       |
| [Parameter](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/parameter.md)                       | 🟢 No loss | 🟢 No loss |       |
| [Variable](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md)                         | 🟢 No loss | 🟢 No loss |       |
| **Style**                                                                                                                 |
| [StyledBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled_block.md)                 | 🟢 No loss | 🟢 No loss |       |
| [StyledInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled_inline.md)               | 🟢 No loss | 🟢 No loss |       |
| **Edits**                                                                                                                 |
| [DeleteBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete_block.md)                 | 🟢 No loss | 🟢 No loss |       |
| [DeleteInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete_inline.md)               | 🟢 No loss | 🟢 No loss |       |
| [InsertBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert_block.md)                 | 🟢 No loss | 🟢 No loss |       |
| [InsertInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert_inline.md)               | 🟢 No loss | 🟢 No loss |       |
| [InstructionBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction_block.md)       | 🟢 No loss | 🟢 No loss |       |
| [InstructionInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction_inline.md)     | 🟢 No loss | 🟢 No loss |       |
| [InstructionMessage](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction_message.md)   | 🟢 No loss | 🟢 No loss |       |
| [ModifyBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify_block.md)                 | 🟢 No loss | 🟢 No loss |       |
| [ModifyInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify_inline.md)               | 🟢 No loss | 🟢 No loss |       |
| [ModifyOperation](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify_operation.md)         | 🟢 No loss | 🟢 No loss |       |
| [ReplaceBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/replace_block.md)               | 🟢 No loss | 🟢 No loss |       |
| [ReplaceInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/replace_inline.md)             | 🟢 No loss | 🟢 No loss |       |
| [StringOperation](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/string_operation.md)         | 🟢 No loss | 🟢 No loss |       |
| [StringPatch](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/string_patch.md)                 | 🟢 No loss | 🟢 No loss |       |
| [SuggestionBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion_block.md)         | 🟢 No loss | 🟢 No loss |       |
| [SuggestionInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion_inline.md)       | 🟢 No loss | 🟢 No loss |       |
| **Other**                                                                                                                 |
| [Brand](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/brand.md)                              | 🟢 No loss | 🟢 No loss |       |
| [ContactPoint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/contact_point.md)               | 🟢 No loss | 🟢 No loss |       |
| [Enumeration](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/enumeration.md)                  | 🟢 No loss | 🟢 No loss |       |
| [Grant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/grant.md)                              | 🟢 No loss | 🟢 No loss |       |
| [InstructionModel](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/instruction_model.md)       | 🟢 No loss | 🟢 No loss |       |
| [MonetaryGrant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/monetary_grant.md)             | 🟢 No loss | 🟢 No loss |       |
| [Organization](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md)                | 🟢 No loss | 🟢 No loss |       |
| [Person](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md)                            | 🟢 No loss | 🟢 No loss |       |
| [PostalAddress](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/postal_address.md)             | 🟢 No loss | 🟢 No loss |       |
| [Product](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/product.md)                          | 🟢 No loss | 🟢 No loss |       |
| [PropertyValue](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property_value.md)             | 🟢 No loss | 🟢 No loss |       |
| [ProvenanceCount](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance_count.md)         | 🟢 No loss | 🟢 No loss |       |
| [Thing](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)                              | 🟢 No loss | 🟢 No loss |       |


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
