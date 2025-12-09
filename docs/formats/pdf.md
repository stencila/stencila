---
title: PDF
description: Portable Document Format
config:
  publish:
    ghost:
      slug: pdf
      tags:
        - "#docs"
        - Formats
---

# Introduction

[Portable Document Format (PDF)](https://pdfa.org/resource/pdf-specification-archive/) is an output format that can be used to render documents suitable for publication or sharing.

# Usage

> [!info]
> Converting to PDF requires [Pandoc](https://pandoc.org/installing.html) and a PDF engine (e.g. `pdflatex`) to be installed.

Use the `.pdf` file extension, or the `--to pdf` option, when converting to PDF e.g.

```sh
stencila convert doc.smd doc.pdf
```

> [!warning]
> Stencila's PDF support is in beta status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Stencila provides lossy bidirectional conversion to PDF is powered by [Pandoc](https://pandoc.org/). To convert documents to PDF, you will need to have Pandoc and a PDF engine installed. See the [`pandoc`](../pandoc) format for more details.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

# Support

Stencila supports these operations for PDF:

- decoding from a file
- encoding to a file

Support and degree of loss by node type:

| Node type                                                                                    | Encoding     | Decoding     | Notes |
| -------------------------------------------------------------------------------------------- | ------------ | ------------ | ----- |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | ⚠️ High loss | ⚠️ High loss |       |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | ⚠️ High loss | ⚠️ High loss |       |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    | ⚠️ High loss | ⚠️ High loss |       |
| [Bibliography](https://stencila.ghost.io/docs/reference/schema/bibliography)                 | ⚠️ High loss | ⚠️ High loss |       |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  | ⚠️ High loss | ⚠️ High loss |       |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       | ⚠️ High loss | ⚠️ High loss |       |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | ⚠️ High loss | ⚠️ High loss |       |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     | ⚠️ High loss | ⚠️ High loss |       |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           | ⚠️ High loss | ⚠️ High loss |       |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                | ⚠️ High loss | ⚠️ High loss |       |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       | ⚠️ High loss | ⚠️ High loss |       |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | ⚠️ High loss | ⚠️ High loss |       |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | ⚠️ High loss | ⚠️ High loss |       |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  | ⚠️ High loss | ⚠️ High loss |       |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     | ⚠️ High loss | ⚠️ High loss |       |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             | ⚠️ High loss | ⚠️ High loss |       |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        | ⚠️ High loss | ⚠️ High loss |       |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      | ⚠️ High loss | ⚠️ High loss |       |
| [Reference](https://stencila.ghost.io/docs/reference/schema/reference)                       | ⚠️ High loss | ⚠️ High loss |       |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             | ⚠️ High loss | ⚠️ High loss |       |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  | ⚠️ High loss | ⚠️ High loss |       |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   | ⚠️ High loss | ⚠️ High loss |       |
| [Supplement](https://stencila.ghost.io/docs/reference/schema/supplement)                     | ⚠️ High loss | ⚠️ High loss |       |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               | ⚠️ High loss | ⚠️ High loss |       |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      | ⚠️ High loss | ⚠️ High loss |       |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        | ⚠️ High loss | ⚠️ High loss |       |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | ⚠️ High loss | ⚠️ High loss |       |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | ⚠️ High loss | ⚠️ High loss |       |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     | ⚠️ High loss | ⚠️ High loss |       |
| [AppendixBreak](https://stencila.ghost.io/docs/reference/schema/appendix_break)              | ⚠️ High loss | ⚠️ High loss |       |
| [Citation](https://stencila.ghost.io/docs/reference/schema/citation)                         | ⚠️ High loss | ⚠️ High loss |       |
| [CitationGroup](https://stencila.ghost.io/docs/reference/schema/citation_group)              | ⚠️ High loss | ⚠️ High loss |       |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  | ⚠️ High loss | ⚠️ High loss |       |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | ⚠️ High loss | ⚠️ High loss |       |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | ⚠️ High loss | ⚠️ High loss |       |
| [InlinesBlock](https://stencila.ghost.io/docs/reference/schema/inlines_block)                | ⚠️ High loss | ⚠️ High loss |       |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | ⚠️ High loss | ⚠️ High loss |       |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | ⚠️ High loss | ⚠️ High loss |       |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | ⚠️ High loss | ⚠️ High loss |       |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | ⚠️ High loss | ⚠️ High loss |       |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | ⚠️ High loss | ⚠️ High loss |       |
| [Sentence](https://stencila.ghost.io/docs/reference/schema/sentence)                         | ⚠️ High loss | ⚠️ High loss |       |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | ⚠️ High loss | ⚠️ High loss |       |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | ⚠️ High loss | ⚠️ High loss |       |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | ⚠️ High loss | ⚠️ High loss |       |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | ⚠️ High loss | ⚠️ High loss |       |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | ⚠️ High loss | ⚠️ High loss |       |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | ⚠️ High loss | ⚠️ High loss |       |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | ⚠️ High loss | ⚠️ High loss |       |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | ⚠️ High loss | ⚠️ High loss |       |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | ⚠️ High loss | ⚠️ High loss |       |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | ⚠️ High loss | ⚠️ High loss |       |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | ⚠️ High loss | ⚠️ High loss |       |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | ⚠️ High loss | ⚠️ High loss |       |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    | ⚠️ High loss | ⚠️ High loss |       |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        | ⚠️ High loss | ⚠️ High loss |       |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               | ⚠️ High loss | ⚠️ High loss |       |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      | ⚠️ High loss | ⚠️ High loss |       |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            | ⚠️ High loss | ⚠️ High loss |       |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | ⚠️ High loss | ⚠️ High loss |       |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        | ⚠️ High loss | ⚠️ High loss |       |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      | ⚠️ High loss | ⚠️ High loss |       |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       | ⚠️ High loss | ⚠️ High loss |       |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          | ⚠️ High loss | ⚠️ High loss |       |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) | ⚠️ High loss | ⚠️ High loss |       |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              | ⚠️ High loss | ⚠️ High loss |       |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | ⚠️ High loss | ⚠️ High loss |       |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     | ⚠️ High loss | ⚠️ High loss |       |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              | ⚠️ High loss | ⚠️ High loss |       |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | ⚠️ High loss | ⚠️ High loss |       |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      | ⚠️ High loss | ⚠️ High loss |       |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              | ⚠️ High loss | ⚠️ High loss |       |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | ⚠️ High loss | ⚠️ High loss |       |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        | ⚠️ High loss | ⚠️ High loss |       |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | ⚠️ High loss | ⚠️ High loss |       |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          | ⚠️ High loss | ⚠️ High loss |       |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             | ⚠️ High loss | ⚠️ High loss |       |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    | ⚠️ High loss | ⚠️ High loss |       |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | ⚠️ High loss | ⚠️ High loss |       |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    | ⚠️ High loss | ⚠️ High loss |       |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          | ⚠️ High loss | ⚠️ High loss |       |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              | ⚠️ High loss | ⚠️ High loss |       |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | ⚠️ High loss | ⚠️ High loss |       |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    | ⚠️ High loss | ⚠️ High loss |       |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            | ⚠️ High loss | ⚠️ High loss |       |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           | ⚠️ High loss | ⚠️ High loss |       |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | ⚠️ High loss | ⚠️ High loss |       |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             | ⚠️ High loss | ⚠️ High loss |       |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                | ⚠️ High loss | ⚠️ High loss |       |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      | ⚠️ High loss | ⚠️ High loss |       |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                | ⚠️ High loss | ⚠️ High loss |       |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      | ⚠️ High loss | ⚠️ High loss |       |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    | ⚠️ High loss | ⚠️ High loss |       |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  | ⚠️ High loss | ⚠️ High loss |       |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                | ⚠️ High loss | ⚠️ High loss |       |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        | ⚠️ High loss | ⚠️ High loss |       |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         | ⚠️ High loss | ⚠️ High loss |       |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          | ⚠️ High loss | ⚠️ High loss |       |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             | ⚠️ High loss | ⚠️ High loss |       |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                | ⚠️ High loss | ⚠️ High loss |       |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       | ⚠️ High loss | ⚠️ High loss |       |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         | ⚠️ High loss | ⚠️ High loss |       |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   | ⚠️ High loss | ⚠️ High loss |       |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          | ⚠️ High loss | ⚠️ High loss |       |
| **Style**                                                                                    |
| [Page](https://stencila.ghost.io/docs/reference/schema/page)                                 | ⚠️ High loss | ⚠️ High loss |       |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  | ⚠️ High loss | ⚠️ High loss |       |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | ⚠️ High loss | ⚠️ High loss |       |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        | ⚠️ High loss | ⚠️ High loss |       |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      | ⚠️ High loss | ⚠️ High loss |       |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    | ⚠️ High loss | ⚠️ High loss |       |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  | ⚠️ High loss | ⚠️ High loss |       |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          | ⚠️ High loss | ⚠️ High loss |       |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        | ⚠️ High loss | ⚠️ High loss |       |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             | ⚠️ High loss | ⚠️ High loss |       |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               | ⚠️ High loss | ⚠️ High loss |       |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                | ⚠️ High loss | ⚠️ High loss |       |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   | ⚠️ High loss | ⚠️ High loss |       |
| [Excerpt](https://stencila.ghost.io/docs/reference/schema/excerpt)                           | ⚠️ High loss | ⚠️ High loss |       |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               | ⚠️ High loss | ⚠️ High loss |       |
| [Island](https://stencila.ghost.io/docs/reference/schema/island)                             | ⚠️ High loss | ⚠️ High loss |       |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          | ⚠️ High loss | ⚠️ High loss |       |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              | ⚠️ High loss | ⚠️ High loss |       |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | ⚠️ High loss | ⚠️ High loss |       |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             | ⚠️ High loss | ⚠️ High loss |       |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | ⚠️ High loss | ⚠️ High loss |       |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | ⚠️ High loss | ⚠️ High loss |       |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              | ⚠️ High loss | ⚠️ High loss |       |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          | ⚠️ High loss | ⚠️ High loss |       |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        | ⚠️ High loss | ⚠️ High loss |       |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               | ⚠️ High loss | ⚠️ High loss |       |

See the Rust crate [`codec-pdf`](https://github.com/stencila/stencila/tree/main/rust/codec-pdf) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
