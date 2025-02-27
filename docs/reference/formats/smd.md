---
config:
  publish:
    ghost:
      slug: smd-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Stencila Markdown
title: smd
---

# Introduction

**File Extension:** `.smd` - Used when converting or exporting Stencila documents to smd format.

The [Stencila Markdown](docs/smd) format is a serialization format which strives to support as many features of the [Stencila Schema](docs/schema) as possible with special Markdown sytnax.

# Implementation

Stencila support bi-directional conversion between Stencila documents and Stencila Markdown powered by the CommonMark [`markdown`](https://crates.io/crates/markdown) crate.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

# Support

Stencila supports these operations for Stencila Markdown:

- decoding from a file
- decoding from a string
- encoding to a file
- encoding to a string

Support and degree of loss by node type:

| Node type                                                                                    | Encoding     | Decoding   | Notes |
| -------------------------------------------------------------------------------------------- | ------------ | ---------- | ----- |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | 🔷 Low loss   | 🔷 Low loss |       |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | 🔷 Low loss   | 🔷 Low loss |       |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    | ⚠️ High loss |            |       |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 | ⚠️ High loss |            |       |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  | ⚠️ High loss |            |       |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       | ⚠️ High loss |            |       |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | ⚠️ High loss |            |       |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     | ⚠️ High loss |            |       |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           | ⚠️ High loss |            |       |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                | ⚠️ High loss |            |       |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       | ⚠️ High loss |            |       |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | ⚠️ High loss |            |       |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 | ⚠️ High loss |            |       |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | 🔷 Low loss   | 🔷 Low loss |       |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  | 🔷 Low loss   | 🔷 Low loss |       |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     | ⚠️ High loss |            |       |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             | ⚠️ High loss |            |       |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        | ⚠️ High loss |            |       |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      | ⚠️ High loss |            |       |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             | ⚠️ High loss |            |       |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  | ⚠️ High loss |            |       |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   | ⚠️ High loss |            |       |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               | 🔷 Low loss   | 🔷 Low loss |       |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      | 🔷 Low loss   | 🔷 Low loss |       |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        | 🔷 Low loss   | 🔷 Low loss |       |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | ⚠️ High loss |            |       |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | 🟢 No loss    | 🟢 No loss  |       |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     | ⚠️ High loss |            |       |
| [Cite](https://stencila.ghost.io/docs/reference/schema/cite)                                 | ⚠️ High loss |            |       |
| [CiteGroup](https://stencila.ghost.io/docs/reference/schema/cite_group)                      | ⚠️ High loss |            |       |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  | ⚠️ High loss |            |       |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | 🟢 No loss    | 🟢 No loss  |       |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | 🟢 No loss    | 🟢 No loss  |       |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | 🔷 Low loss   | 🔷 Low loss |       |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | 🔷 Low loss   | 🔷 Low loss |       |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | 🔷 Low loss   | 🔷 Low loss |       |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | 🔷 Low loss   | 🔷 Low loss |       |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | 🟢 No loss    | 🟢 No loss  |       |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | 🟢 No loss    | 🟢 No loss  |       |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | ⚠️ High loss |            |       |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | 🟢 No loss    | 🟢 No loss  |       |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | ⚠️ High loss |            |       |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | 🟢 No loss    | 🟢 No loss  |       |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | 🟢 No loss    | 🟢 No loss  |       |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | 🟢 No loss    | 🟢 No loss  |       |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | 🟢 No loss    | 🟢 No loss  |       |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | 🟢 No loss    | 🟢 No loss  |       |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | 🟢 No loss    | 🟢 No loss  |       |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | 🟢 No loss    | 🟢 No loss  |       |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | 🟢 No loss    | 🟢 No loss  |       |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | 🟢 No loss    | 🟢 No loss  |       |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | 🔷 Low loss   | 🔷 Low loss |       |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | 🔷 Low loss   | 🔷 Low loss |       |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | 🟢 No loss    | 🟢 No loss  |       |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    | ⚠️ High loss |            |       |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        | ⚠️ High loss |            |       |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               | ⚠️ High loss |            |       |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      | ⚠️ High loss |            |       |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            | ⚠️ High loss |            |       |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | 🔷 Low loss   | 🔷 Low loss |       |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        | ⚠️ High loss |            |       |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      | ⚠️ High loss |            |       |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | 🟢 No loss    | 🟢 No loss  |       |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       | ⚠️ High loss |            |       |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          | ⚠️ High loss |            |       |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) | ⚠️ High loss |            |       |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              | ⚠️ High loss |            |       |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | ⚠️ High loss |            |       |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | ⚠️ High loss |            |       |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     | ⚠️ High loss |            |       |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              | ⚠️ High loss |            |       |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | ⚠️ High loss |            |       |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      | ⚠️ High loss |            |       |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              | ⚠️ High loss |            |       |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | 🔷 Low loss   | 🔷 Low loss |       |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        | ⚠️ High loss |            |       |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | 🔷 Low loss   | 🔷 Low loss |       |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | 🔷 Low loss   | 🔷 Low loss |       |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          | ⚠️ High loss |            |       |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             | ⚠️ High loss |            |       |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    | ⚠️ High loss |            |       |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | 🟢 No loss    | 🟢 No loss  |       |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    | ⚠️ High loss |            |       |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          | ⚠️ High loss |            |       |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | ⚠️ High loss |            |       |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              | ⚠️ High loss |            |       |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | ⚠️ High loss |            |       |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    | ⚠️ High loss |            |       |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            | ⚠️ High loss |            |       |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           | ⚠️ High loss |            |       |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | 🔷 Low loss   | 🔷 Low loss |       |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             | ⚠️ High loss |            |       |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                | ⚠️ High loss |            |       |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      | ⚠️ High loss |            |       |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                | ⚠️ High loss |            |       |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      | ⚠️ High loss |            |       |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    | ⚠️ High loss |            |       |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  | ⚠️ High loss |            |       |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                | ⚠️ High loss |            |       |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        | ⚠️ High loss |            |       |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 | ⚠️ High loss |            |       |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         | ⚠️ High loss |            |       |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          | ⚠️ High loss |            |       |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             | ⚠️ High loss |            |       |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                | ⚠️ High loss |            |       |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       | 🔷 Low loss   | 🔷 Low loss |       |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         | ⚠️ High loss |            |       |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   | ⚠️ High loss |            |       |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          | ⚠️ High loss |            |       |
| **Style**                                                                                    |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  | 🟢 No loss    | 🟢 No loss  |       |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | ⚠️ High loss |            |       |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        | ⚠️ High loss |            |       |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      | ⚠️ High loss |            |       |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    | ⚠️ High loss |            |       |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  | ⚠️ High loss |            |       |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          | ⚠️ High loss |            |       |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        | ⚠️ High loss |            |       |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             | ⚠️ High loss |            |       |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               | ⚠️ High loss |            |       |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                | ⚠️ High loss |            |       |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   | ⚠️ High loss |            |       |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               | ⚠️ High loss |            |       |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          | ⚠️ High loss |            |       |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              | ⚠️ High loss |            |       |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | ⚠️ High loss |            |       |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             | ⚠️ High loss |            |       |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | ⚠️ High loss |            |       |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | ⚠️ High loss |            |       |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              | ⚠️ High loss |            |       |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          | ⚠️ High loss |            |       |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        | ⚠️ High loss |            |       |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               | ⚠️ High loss |            |       |

See the Rust crate [`codec-markdown`](https://github.com/stencila/stencila/tree/main/rust/codec-markdown) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
