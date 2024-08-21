# Markdown

## Introduction

Markdown is a lightweight markup language widely used for formatting plain text documents. It provides a simple and human-readable way to structure text and add basic styling, such as headers, lists, links, and emphasis. Markdown's benefits include ease of use, and compatibility with various web and documentation platforms.

## Implementation

Stencila support bi-directional conversion between Stencila documents and Markdown.

Three internal Rust crates are involved in the conversion from Stencila documents to Markdown:

- The `codec-markdown` crate defines the `MarkdownCodec` `struct` which implements `to_string` method of the `Codec` `trait` by calling the `to_markdown` method of the `MarkdownCodec` `trait`.

- The `codec-markdown-trait` crate defines the `MarkdownCodec` `trait` which has the `to_markdown` method.

- The `codec-markdown-derive` crate provides a derive macro which is used to derive the `MarkdownCodec` trait for all types in the Stencila Schema.

It is necessary to have three separate crates because of the need to have a separate crate for derive macros and to avoid circular dependencies.

The `MarkdownCodec` derive macro has a `#[markdown(...)]` helper attribute which can be used to specify options for how the `to_markdown` method is derived for a type:

- `template`: A string, compatible with the Rust [`format!` macro](https://doc.rust-lang.org/std/macro.format.html), which specifies how a node will be represented in Markdown

- `escape`: A character that should be escaped (with a single backslash) prior to applying the template

- `special`: A boolean specifying whether a special, manually written function should be used for encoding the type to Markdown. If this is `true` then the type must implement a function named `to_markdown_special` with the same signature as the `to_markdown` method.

These options should be set in the `schema/*.yaml` files. These options will flow through to the `#[markdown(...)]` derive helper for the type when the files in `rust/schema/type` are regenerated. For example, the `schema/Strong.yaml` file contains the following:

```yaml
markdown:
  template: '**{content}**'
  escape: '*'
```

And the `schema/Heading.yaml` file contains the following:

```yaml
markdown:
  special: true
```

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

## Codec

The codec (en**co**der/**dec**oder) for Markdown supports:

- decoding from a file
- decoding from a string
- encoding to a file
- encoding to a string

Support and degree of loss for node types:

| Node type                                                                                                                 | Encoding     | Decoding   | Notes                                                 |
| ------------------------------------------------------------------------------------------------------------------------- | ------------ | ---------- | ----------------------------------------------------- |
| **Works**                                                                                                                 |
| [Article](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/article.md)                          | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [AudioObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/audio_object.md)                 | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [AuthorRole](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author_role.md)                   | ⚠️ High loss |            |                                                       |
| [Claim](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/claim.md)                              | ⚠️ High loss |            | Encoded using implemented function                    |
| [Collection](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/collection.md)                    | ⚠️ High loss |            |                                                       |
| [Comment](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/comment.md)                          | ⚠️ High loss |            |                                                       |
| [CreativeWork](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/creative_work.md)               | ⚠️ High loss |            |                                                       |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/directory.md)                      | ⚠️ High loss |            |                                                       |
| [Figure](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/figure.md)                            | ⚠️ High loss |            | Encoded using implemented function                    |
| [File](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/file.md)                                | ⚠️ High loss |            |                                                       |
| [ImageObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image_object.md)                 | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [MediaObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/media_object.md)                 | 🔷 Low loss   | 🔷 Low loss |                                                       |
| [Periodical](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/periodical.md)                    | ⚠️ High loss |            |                                                       |
| [Prompt](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/prompt.md)                            | ⚠️ High loss |            |                                                       |
| [PublicationIssue](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/publication_issue.md)       | ⚠️ High loss |            |                                                       |
| [PublicationVolume](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/publication_volume.md)     | ⚠️ High loss |            |                                                       |
| [Review](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/review.md)                            | ⚠️ High loss |            |                                                       |
| [SoftwareApplication](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software_application.md) | ⚠️ High loss |            |                                                       |
| [SoftwareSourceCode](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software_source_code.md)  | ⚠️ High loss |            |                                                       |
| [Table](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table.md)                              | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [TableCell](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table_cell.md)                     | 🔷 Low loss   | 🔷 Low loss |                                                       |
| [TableRow](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table_row.md)                       | 🔷 Low loss   | 🔷 Low loss |                                                       |
| [VideoObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/video_object.md)                 | ⚠️ High loss |            | Encoded using implemented function                    |
| **Prose**                                                                                                                 |
| [Admonition](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md)                    | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [Cite](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)                                | ⚠️ High loss |            |                                                       |
| [CiteGroup](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite_group.md)                     | ⚠️ High loss |            |                                                       |
| [DefinedTerm](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/defined_term.md)                 | ⚠️ High loss |            |                                                       |
| [Emphasis](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/emphasis.md)                        | 🟢 No loss    | 🟢 No loss  | Encoded as `_{{content}}_`                            |
| [Heading](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md)                          | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [Link](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)                                | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [List](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md)                                | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [ListItem](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list_item.md)                       | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [Note](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/note.md)                                | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [Paragraph](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md)                      | 🟢 No loss    | 🟢 No loss  | Encoded as `{{content}}\n\n`                          |
| [QuoteBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote_block.md)                   | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [QuoteInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote_inline.md)                 | ⚠️ High loss |            | Encoded as `<q>{{content}}</q>`                       |
| [Section](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md)                          | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [Strikeout](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strikeout.md)                      | ⚠️ High loss |            | Encoded as `~~{{content}}~~`                          |
| [Strong](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strong.md)                            | 🟢 No loss    | 🟢 No loss  | Encoded as `**{{content}}**`                          |
| [Subscript](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/subscript.md)                      | 🟢 No loss    | 🟢 No loss  | Encoded as `~{{content}}~`                            |
| [Superscript](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/superscript.md)                  | 🟢 No loss    | 🟢 No loss  | Encoded as `^{{content}}^`                            |
| [Text](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)                                | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [ThematicBreak](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/thematic_break.md)             | 🟢 No loss    | 🟢 No loss  | Encoded as `***\n\n`                                  |
| [Underline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/underline.md)                      | 🟢 No loss    | 🟢 No loss  | Encoded as `<u>{{content}}</u>`                       |
| **Math**                                                                                                                  |
| [MathBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math_block.md)                      | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [MathInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math_inline.md)                    | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| **Code**                                                                                                                  |
| [CodeBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_block.md)                      | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [CodeChunk](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_chunk.md)                      | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [CodeExpression](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_expression.md)            | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [CodeInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_inline.md)                    | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [CompilationMessage](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation_message.md)    | ⚠️ High loss |            |                                                       |
| [ExecutionMessage](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution_message.md)        | ⚠️ High loss |            |                                                       |
| **Data**                                                                                                                  |
| [Array](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array.md)                               | ⚠️ High loss |            |                                                       |
| [ArrayHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array_hint.md)                      | ⚠️ High loss |            |                                                       |
| [ArrayValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array_validator.md)            | ⚠️ High loss |            | Encoded using implemented function                    |
| [Boolean](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)                           | 🔷 Low loss   | 🔷 Low loss |                                                       |
| [BooleanValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean_validator.md)        | ⚠️ High loss |            | Encoded using implemented function                    |
| [ConstantValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/constant_validator.md)      | ⚠️ High loss |            | Encoded using implemented function                    |
| [Cord](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                                 | 🟢 No loss    | 🟢 No loss  |                                                       |
| [Datatable](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable.md)                       | ⚠️ High loss |            | Encoded using implemented function                    |
| [DatatableColumn](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable_column.md)          | ⚠️ High loss |            |                                                       |
| [DatatableColumnHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable_column_hint.md) | ⚠️ High loss |            |                                                       |
| [DatatableHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable_hint.md)              | ⚠️ High loss |            |                                                       |
| [Date](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)                                 | ⚠️ High loss |            |                                                       |
| [DateTime](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_time.md)                        | ⚠️ High loss |            |                                                       |
| [DateTimeValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_time_validator.md)     | ⚠️ High loss |            | Encoded using implemented function                    |
| [DateValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_validator.md)              | ⚠️ High loss |            | Encoded using implemented function                    |
| [Duration](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                         | ⚠️ High loss |            |                                                       |
| [DurationValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration_validator.md)      | ⚠️ High loss |            | Encoded using implemented function                    |
| [EnumValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/enum_validator.md)              | ⚠️ High loss |            | Encoded using implemented function                    |
| [Integer](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                           | 🔷 Low loss   | 🔷 Low loss |                                                       |
| [IntegerValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer_validator.md)        | ⚠️ High loss |            | Encoded using implemented function                    |
| [Null](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/null.md)                                 | 🔷 Low loss   | 🔷 Low loss |                                                       |
| [Number](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md)                             | 🔷 Low loss   | 🔷 Low loss |                                                       |
| [NumberValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number_validator.md)          | ⚠️ High loss |            | Encoded using implemented function                    |
| [Object](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/object.md)                             | ⚠️ High loss |            |                                                       |
| [ObjectHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/object_hint.md)                    | ⚠️ High loss |            |                                                       |
| [String](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | 🟢 No loss    | 🟢 No loss  |                                                       |
| [StringHint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string_hint.md)                    | ⚠️ High loss |            |                                                       |
| [StringValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string_validator.md)          | ⚠️ High loss |            | Encoded using implemented function                    |
| [Time](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)                                 | ⚠️ High loss |            |                                                       |
| [TimeValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time_validator.md)              | ⚠️ High loss |            | Encoded using implemented function                    |
| [Timestamp](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                       | ⚠️ High loss |            |                                                       |
| [TimestampValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp_validator.md)    | ⚠️ High loss |            | Encoded using implemented function                    |
| [TupleValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/tuple_validator.md)            | ⚠️ High loss |            | Encoded using implemented function                    |
| [Unknown](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unknown.md)                           | ⚠️ High loss |            |                                                       |
| [UnsignedInteger](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned_integer.md)          | 🔷 Low loss   | 🔷 Low loss |                                                       |
| **Flow**                                                                                                                  |
| [Button](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/button.md)                             | ⚠️ High loss |            |                                                       |
| [CallArgument](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call_argument.md)                | ⚠️ High loss |            | Encoded using implemented function                    |
| [CallBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call_block.md)                      | ⚠️ High loss |            | Encoded using implemented function                    |
| [CodeLocation](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/code_location.md)                | ⚠️ High loss |            |                                                       |
| [CompilationDigest](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation_digest.md)      | ⚠️ High loss |            |                                                       |
| [ExecutionDependant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_dependant.md)    | ⚠️ High loss |            |                                                       |
| [ExecutionDependency](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_dependency.md)  | ⚠️ High loss |            |                                                       |
| [ExecutionTag](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_tag.md)                | ⚠️ High loss |            |                                                       |
| [ForBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for_block.md)                        | ⚠️ High loss |            | Encoded using implemented function                    |
| [Form](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)                                 | ⚠️ High loss |            |                                                       |
| [Function](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/function.md)                         | ⚠️ High loss |            |                                                       |
| [IfBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if_block.md)                          | ⚠️ High loss |            | Encoded using implemented function                    |
| [IfBlockClause](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if_block_clause.md)             | ⚠️ High loss |            | Encoded using implemented function                    |
| [IncludeBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include_block.md)                | ⚠️ High loss |            | Encoded using implemented function                    |
| [Parameter](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/parameter.md)                       | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function                    |
| [Variable](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md)                         | ⚠️ High loss |            |                                                       |
| **Style**                                                                                                                 |
| [StyledBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled_block.md)                 | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function                    |
| [StyledInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled_inline.md)               | ⚠️ High loss |            | Encoded using implemented function                    |
| **Edits**                                                                                                                 |
| [DeleteBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete_block.md)                 | ⚠️ High loss |            | Encoded using implemented function                    |
| [DeleteInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/delete_inline.md)               | ⚠️ High loss |            | Encoded as `[[delete {{content}}]]`                   |
| [InsertBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert_block.md)                 | ⚠️ High loss |            | Encoded using implemented function                    |
| [InsertInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/insert_inline.md)               | ⚠️ High loss |            | Encoded as `[[insert {{content}}]]`                   |
| [InstructionBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction_block.md)       | ⚠️ High loss |            | Encoded using implemented function                    |
| [InstructionInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction_inline.md)     | ⚠️ High loss |            | Encoded using implemented function                    |
| [InstructionMessage](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction_message.md)   | ⚠️ High loss |            | Encoded using implemented function                    |
| [ModifyBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify_block.md)                 | ⚠️ High loss |            | Encoded using implemented function                    |
| [ModifyInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify_inline.md)               | ⚠️ High loss |            | Encoded using implemented function                    |
| [ModifyOperation](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/modify_operation.md)         | ⚠️ High loss |            |                                                       |
| [ReplaceBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/replace_block.md)               | ⚠️ High loss |            | Encoded using implemented function                    |
| [ReplaceInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/replace_inline.md)             | ⚠️ High loss |            | Encoded as `[[replace {{content}}>>{{replacement}}]]` |
| [StringOperation](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/string_operation.md)         | ⚠️ High loss |            |                                                       |
| [StringPatch](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/string_patch.md)                 | ⚠️ High loss |            |                                                       |
| [SuggestionBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion_block.md)         | ⚠️ High loss |            | Encoded using implemented function                    |
| [SuggestionInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion_inline.md)       | ⚠️ High loss |            | Encoded using implemented function                    |
| **Other**                                                                                                                 |
| [Brand](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/brand.md)                              | ⚠️ High loss |            |                                                       |
| [ContactPoint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/contact_point.md)               | ⚠️ High loss |            |                                                       |
| [Enumeration](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/enumeration.md)                  | ⚠️ High loss |            |                                                       |
| [Grant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/grant.md)                              | ⚠️ High loss |            |                                                       |
| [InstructionModel](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/instruction_model.md)       | ⚠️ High loss |            |                                                       |
| [MonetaryGrant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/monetary_grant.md)             | ⚠️ High loss |            |                                                       |
| [Organization](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md)                | ⚠️ High loss |            |                                                       |
| [Person](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md)                            | ⚠️ High loss |            |                                                       |
| [PostalAddress](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/postal_address.md)             | ⚠️ High loss |            |                                                       |
| [Product](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/product.md)                          | ⚠️ High loss |            |                                                       |
| [PropertyValue](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property_value.md)             | ⚠️ High loss |            |                                                       |
| [ProvenanceCount](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/provenance_count.md)         | ⚠️ High loss |            |                                                       |
| [Thing](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)                              | ⚠️ High loss |            |                                                       |


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
