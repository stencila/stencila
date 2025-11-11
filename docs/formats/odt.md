---
title: ODT
description: Open Document Text
config:
  publish:
    ghost:
      slug: odt
      tags:
        - "#docs"
        - Formats
---

# Introduction

The [Open Document Text (ODT)](https://docs.oasis-open.org/office/v1.1/OS/OpenDocument-v1.1-html/OpenDocument-v1.1.html) format is a serialization format is a convenient when you want to pass open documents to and from word processors.

# Usage

> [!info]
> Converting to/from ODT requires [Pandoc to be installed](https://pandoc.org/installing.html).

Use the `.odt` file extension, or the `--to odt` or `--from odt` options, when converting to/from ODT e.g.

```sh
stencila convert doc.smd doc.odt
```

> [!warning]
> Stencila's ODT support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila provides lossy bidirectional conversion to ODT powered by [Pandoc](https://pandoc.org/). To convert documents to/from ODT, you will need to have Pandoc installed. See the [`pandoc`](../pandoc) format for more details.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

# Support

Stencila supports these operations for OpenDocument Text:

- decoding from a file
- encoding to a file

Support and degree of loss by node type:

| Node type                                                                                    | Encoding   | Decoding   | Notes |
| -------------------------------------------------------------------------------------------- | ---------- | ---------- | ----- |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | 🔷 Low loss | 🔷 Low loss |       |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | 🔷 Low loss | 🔷 Low loss |       |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    | 🔷 Low loss | 🔷 Low loss |       |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  | 🔷 Low loss | 🔷 Low loss |       |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       | 🔷 Low loss | 🔷 Low loss |       |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | 🔷 Low loss | 🔷 Low loss |       |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     | 🔷 Low loss | 🔷 Low loss |       |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           | 🔷 Low loss | 🔷 Low loss |       |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                | 🔷 Low loss | 🔷 Low loss |       |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       | 🔷 Low loss | 🔷 Low loss |       |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | 🔷 Low loss | 🔷 Low loss |       |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | 🔷 Low loss | 🔷 Low loss |       |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  | 🔷 Low loss | 🔷 Low loss |       |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     | 🔷 Low loss | 🔷 Low loss |       |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             | 🔷 Low loss | 🔷 Low loss |       |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        | 🔷 Low loss | 🔷 Low loss |       |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      | 🔷 Low loss | 🔷 Low loss |       |
| [Reference](https://stencila.ghost.io/docs/reference/schema/reference)                       | 🔷 Low loss | 🔷 Low loss |       |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             | 🔷 Low loss | 🔷 Low loss |       |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  | 🔷 Low loss | 🔷 Low loss |       |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   | 🔷 Low loss | 🔷 Low loss |       |
| [Supplement](https://stencila.ghost.io/docs/reference/schema/supplement)                     | 🔷 Low loss | 🔷 Low loss |       |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               | 🔷 Low loss | 🔷 Low loss |       |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      | 🔷 Low loss | 🔷 Low loss |       |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        | 🔷 Low loss | 🔷 Low loss |       |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | 🔷 Low loss | 🔷 Low loss |       |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | 🔷 Low loss | 🔷 Low loss |       |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     | 🔷 Low loss | 🔷 Low loss |       |
| [AppendixBreak](https://stencila.ghost.io/docs/reference/schema/appendix_break)              | 🔷 Low loss | 🔷 Low loss |       |
| [Citation](https://stencila.ghost.io/docs/reference/schema/citation)                         | 🔷 Low loss | 🔷 Low loss |       |
| [CitationGroup](https://stencila.ghost.io/docs/reference/schema/citation_group)              | 🔷 Low loss | 🔷 Low loss |       |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  | 🔷 Low loss | 🔷 Low loss |       |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | 🔷 Low loss | 🔷 Low loss |       |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | 🔷 Low loss | 🔷 Low loss |       |
| [InlinesBlock](https://stencila.ghost.io/docs/reference/schema/inlines_block)                | 🔷 Low loss | 🔷 Low loss |       |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | 🔷 Low loss | 🔷 Low loss |       |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | 🔷 Low loss | 🔷 Low loss |       |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | 🔷 Low loss | 🔷 Low loss |       |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | 🔷 Low loss | 🔷 Low loss |       |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | 🔷 Low loss | 🔷 Low loss |       |
| [Sentence](https://stencila.ghost.io/docs/reference/schema/sentence)                         | 🔷 Low loss | 🔷 Low loss |       |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | 🔷 Low loss | 🔷 Low loss |       |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | 🔷 Low loss | 🔷 Low loss |       |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | 🔷 Low loss | 🔷 Low loss |       |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | 🔷 Low loss | 🔷 Low loss |       |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | 🔷 Low loss | 🔷 Low loss |       |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | 🔷 Low loss | 🔷 Low loss |       |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | 🔷 Low loss | 🔷 Low loss |       |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | 🔷 Low loss | 🔷 Low loss |       |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | 🔷 Low loss | 🔷 Low loss |       |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | 🔷 Low loss | 🔷 Low loss |       |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | 🔷 Low loss | 🔷 Low loss |       |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | 🔷 Low loss | 🔷 Low loss |       |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    | 🔷 Low loss | 🔷 Low loss |       |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        | 🔷 Low loss | 🔷 Low loss |       |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               | 🔷 Low loss | 🔷 Low loss |       |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      | 🔷 Low loss | 🔷 Low loss |       |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            | 🔷 Low loss | 🔷 Low loss |       |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | 🔷 Low loss | 🔷 Low loss |       |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        | 🔷 Low loss | 🔷 Low loss |       |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      | 🔷 Low loss | 🔷 Low loss |       |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       | 🔷 Low loss | 🔷 Low loss |       |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          | 🔷 Low loss | 🔷 Low loss |       |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) | 🔷 Low loss | 🔷 Low loss |       |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              | 🔷 Low loss | 🔷 Low loss |       |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | 🔷 Low loss | 🔷 Low loss |       |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     | 🔷 Low loss | 🔷 Low loss |       |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              | 🔷 Low loss | 🔷 Low loss |       |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | 🔷 Low loss | 🔷 Low loss |       |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      | 🔷 Low loss | 🔷 Low loss |       |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              | 🔷 Low loss | 🔷 Low loss |       |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | 🔷 Low loss | 🔷 Low loss |       |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        | 🔷 Low loss | 🔷 Low loss |       |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | 🔷 Low loss | 🔷 Low loss |       |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          | 🔷 Low loss | 🔷 Low loss |       |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             | 🔷 Low loss | 🔷 Low loss |       |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    | 🔷 Low loss | 🔷 Low loss |       |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | 🔷 Low loss | 🔷 Low loss |       |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    | 🔷 Low loss | 🔷 Low loss |       |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          | 🔷 Low loss | 🔷 Low loss |       |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              | 🔷 Low loss | 🔷 Low loss |       |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | 🔷 Low loss | 🔷 Low loss |       |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    | 🔷 Low loss | 🔷 Low loss |       |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            | 🔷 Low loss | 🔷 Low loss |       |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           | 🔷 Low loss | 🔷 Low loss |       |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | 🔷 Low loss | 🔷 Low loss |       |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             | 🔷 Low loss | 🔷 Low loss |       |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                | 🔷 Low loss | 🔷 Low loss |       |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      | 🔷 Low loss | 🔷 Low loss |       |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                | 🔷 Low loss | 🔷 Low loss |       |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      | 🔷 Low loss | 🔷 Low loss |       |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    | 🔷 Low loss | 🔷 Low loss |       |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  | 🔷 Low loss | 🔷 Low loss |       |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                | 🔷 Low loss | 🔷 Low loss |       |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        | 🔷 Low loss | 🔷 Low loss |       |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         | 🔷 Low loss | 🔷 Low loss |       |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          | 🔷 Low loss | 🔷 Low loss |       |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             | 🔷 Low loss | 🔷 Low loss |       |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                | 🔷 Low loss | 🔷 Low loss |       |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       | 🔷 Low loss | 🔷 Low loss |       |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         | 🔷 Low loss | 🔷 Low loss |       |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   | 🔷 Low loss | 🔷 Low loss |       |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          | 🔷 Low loss | 🔷 Low loss |       |
| **Style**                                                                                    |
| [Page](https://stencila.ghost.io/docs/reference/schema/page)                                 | 🔷 Low loss | 🔷 Low loss |       |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  | 🔷 Low loss | 🔷 Low loss |       |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | 🔷 Low loss | 🔷 Low loss |       |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        | 🔷 Low loss | 🔷 Low loss |       |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      | 🔷 Low loss | 🔷 Low loss |       |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    | 🔷 Low loss | 🔷 Low loss |       |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  | 🔷 Low loss | 🔷 Low loss |       |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          | 🔷 Low loss | 🔷 Low loss |       |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        | 🔷 Low loss | 🔷 Low loss |       |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             | 🔷 Low loss | 🔷 Low loss |       |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               | 🔷 Low loss | 🔷 Low loss |       |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                | 🔷 Low loss | 🔷 Low loss |       |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   | 🔷 Low loss | 🔷 Low loss |       |
| [Excerpt](https://stencila.ghost.io/docs/reference/schema/excerpt)                           | 🔷 Low loss | 🔷 Low loss |       |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               | 🔷 Low loss | 🔷 Low loss |       |
| [Island](https://stencila.ghost.io/docs/reference/schema/island)                             | 🔷 Low loss | 🔷 Low loss |       |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          | 🔷 Low loss | 🔷 Low loss |       |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              | 🔷 Low loss | 🔷 Low loss |       |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | 🔷 Low loss | 🔷 Low loss |       |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             | 🔷 Low loss | 🔷 Low loss |       |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | 🔷 Low loss | 🔷 Low loss |       |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | 🔷 Low loss | 🔷 Low loss |       |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              | 🔷 Low loss | 🔷 Low loss |       |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          | 🔷 Low loss | 🔷 Low loss |       |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        | 🔷 Low loss | 🔷 Low loss |       |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               | 🔷 Low loss | 🔷 Low loss |       |

See the Rust crate [`codec-odt`](https://github.com/stencila/stencila/tree/main/rust/codec-odt) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
