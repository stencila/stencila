---
title: HTML
description: HyperText Markup Language
config:
  publish:
    ghost:
      slug: html
      tags:
        - "#docs"
        - Formats
---

# Introduction

HTML (HyperText Markup Language) is the standard markup language used to structure and display content on the web

Stencila provides, currently limited, support for converting to, and from, HTML.

> [!note] When previewing documents (e.g. using `stencila preview` or in our VSCode extension) Stencila uses a specific encoding of HTML, distinct to the one described here, which we call "DOM HTML". I has very low loss and includes custom elements for Stencila node types (e.g. `<stencila-paragraph>` and `<stencila-codec-chunk`>) but as a result is intentionally verbose.

# Usage

Use the `.html` file extension, or the `--to html` or `--from html` options, when converting to/from HTML e.g.

```sh
stencila convert doc.smd doc.html
```

> [!warning]
> Stencila's HTML support is in alpha status. If you find bugs or unexpected results please [file an issue](https://github.com/stencila/stencila/issues/new).

# Implementation

Parsing of HTML is largely done using the [quick-xml](https://crates.io/crates/quick-xml) Rust crate.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

# Support

Stencila supports these operations for HTML:

- decoding from a file
- decoding from a string
- encoding to a file
- encoding to a string

Support and degree of loss by node type:

| Node type                                                                                    | Encoding  | Decoding | Notes                                                                                                              |
| -------------------------------------------------------------------------------------------- | --------- | -------- | ------------------------------------------------------------------------------------------------------------------ |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | 🟢 No loss |          | Encoded as [`<article>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/article)                        |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | 🟢 No loss |          | Encoded as [`<audio>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio) using special function     |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    | 🟢 No loss |          |                                                                                                                    |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 | 🟢 No loss |          |                                                                                                                    |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  | 🟢 No loss |          |                                                                                                                    |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       | 🟢 No loss |          |                                                                                                                    |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | 🟢 No loss |          |                                                                                                                    |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     | 🟢 No loss |          |                                                                                                                    |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           | 🟢 No loss |          |                                                                                                                    |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                | 🟢 No loss |          |                                                                                                                    |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       | 🟢 No loss |          |                                                                                                                    |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | 🟢 No loss |          | Encoded as [`<figure>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure)                          |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 | 🟢 No loss |          |                                                                                                                    |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | 🟢 No loss |          | Encoded as [`<img>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img) using special function         |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  | 🟢 No loss |          |                                                                                                                    |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     | 🟢 No loss |          |                                                                                                                    |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             | 🟢 No loss |          |                                                                                                                    |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        | 🟢 No loss |          |                                                                                                                    |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      | 🟢 No loss |          |                                                                                                                    |
| [Reference](https://stencila.ghost.io/docs/reference/schema/reference)                       | 🟢 No loss |          |                                                                                                                    |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             | 🟢 No loss |          |                                                                                                                    |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  | 🟢 No loss |          |                                                                                                                    |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   | 🟢 No loss |          |                                                                                                                    |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               | 🟢 No loss |          | Encoded using special function                                                                                     |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      | 🟢 No loss |          | Encoded as [`<td>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)                                  |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        | 🟢 No loss |          | Encoded as [`<tr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr)                                  |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | 🟢 No loss |          | Encoded as [`<video>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video) using special function     |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | 🟢 No loss |          | Encoded as [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)                            |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     | 🟢 No loss |          |                                                                                                                    |
| [AppendixBreak](https://stencila.ghost.io/docs/reference/schema/appendix_break)              | 🟢 No loss |          |                                                                                                                    |
| [Citation](https://stencila.ghost.io/docs/reference/schema/citation)                         | 🟢 No loss |          |                                                                                                                    |
| [CitationGroup](https://stencila.ghost.io/docs/reference/schema/citation_group)              | 🟢 No loss |          |                                                                                                                    |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  | 🟢 No loss |          |                                                                                                                    |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | 🟢 No loss |          | Encoded as [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)                                  |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | 🟢 No loss |          | Encoded using special function                                                                                     |
| [InlinesBlock](https://stencila.ghost.io/docs/reference/schema/inlines_block)                | 🟢 No loss |          |                                                                                                                    |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | 🟢 No loss |          | Encoded as [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)                                    |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | 🟢 No loss |          | Encoded using special function                                                                                     |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | 🟢 No loss |          | Encoded as [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)                                  |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | 🟢 No loss |          |                                                                                                                    |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | 🟢 No loss |          | Encoded as [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)                                    |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | 🟢 No loss |          | Encoded as [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote)                  |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | 🟢 No loss |          | Encoded as [`<q>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)                                    |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | 🟢 No loss |          | Encoded as [`<section>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section) using special function |
| [Sentence](https://stencila.ghost.io/docs/reference/schema/sentence)                         | 🟢 No loss |          |                                                                                                                    |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | 🟢 No loss |          | Encoded as [`<s>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s)                                    |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | 🟢 No loss |          | Encoded as [`<strong>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong)                          |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | 🟢 No loss |          | Encoded as [`<sub>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub)                                |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | 🟢 No loss |          | Encoded as [`<sup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup)                                |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | 🟢 No loss |          | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)                              |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | 🟢 No loss |          | Encoded as [`<hr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)                                  |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | 🟢 No loss |          | Encoded as [`<u>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u)                                    |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | 🟢 No loss |          | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                              |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | 🟢 No loss |          | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                              |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | 🟢 No loss |          | Encoded as [`<pre>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)                                |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | 🟢 No loss |          |                                                                                                                    |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | 🟢 No loss |          |                                                                                                                    |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | 🟢 No loss |          | Encoded as [`<code>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code)                              |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    | 🟢 No loss |          |                                                                                                                    |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        | 🟢 No loss |          |                                                                                                                    |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               | 🟢 No loss |          |                                                                                                                    |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      | 🟢 No loss |          |                                                                                                                    |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            | 🟢 No loss |          |                                                                                                                    |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | 🟢 No loss |          |                                                                                                                    |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        | 🟢 No loss |          |                                                                                                                    |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      | 🟢 No loss |          |                                                                                                                    |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | 🟢 No loss |          |                                                                                                                    |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       | 🟢 No loss |          |                                                                                                                    |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          | 🟢 No loss |          |                                                                                                                    |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) | 🟢 No loss |          |                                                                                                                    |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              | 🟢 No loss |          |                                                                                                                    |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | 🟢 No loss |          |                                                                                                                    |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | 🟢 No loss |          |                                                                                                                    |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     | 🟢 No loss |          |                                                                                                                    |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              | 🟢 No loss |          |                                                                                                                    |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | 🟢 No loss |          |                                                                                                                    |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      | 🟢 No loss |          |                                                                                                                    |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              | 🟢 No loss |          |                                                                                                                    |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | 🟢 No loss |          |                                                                                                                    |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        | 🟢 No loss |          |                                                                                                                    |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | 🟢 No loss |          |                                                                                                                    |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | 🟢 No loss |          |                                                                                                                    |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          | 🟢 No loss |          |                                                                                                                    |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             | 🟢 No loss |          |                                                                                                                    |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    | 🟢 No loss |          |                                                                                                                    |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | 🟢 No loss |          |                                                                                                                    |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    | 🟢 No loss |          |                                                                                                                    |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          | 🟢 No loss |          |                                                                                                                    |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | 🟢 No loss |          |                                                                                                                    |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              | 🟢 No loss |          |                                                                                                                    |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | 🟢 No loss |          |                                                                                                                    |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    | 🟢 No loss |          |                                                                                                                    |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            | 🟢 No loss |          |                                                                                                                    |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           | 🟢 No loss |          |                                                                                                                    |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | 🟢 No loss |          |                                                                                                                    |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             | 🟢 No loss |          | Encoded as [`<button>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button)                          |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                | 🟢 No loss |          |                                                                                                                    |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      | 🟢 No loss |          |                                                                                                                    |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                | 🟢 No loss |          |                                                                                                                    |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      | 🟢 No loss |          |                                                                                                                    |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    | 🟢 No loss |          |                                                                                                                    |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  | 🟢 No loss |          |                                                                                                                    |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                | 🟢 No loss |          |                                                                                                                    |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        | 🟢 No loss |          |                                                                                                                    |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 | 🟢 No loss |          |                                                                                                                    |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         | 🟢 No loss |          |                                                                                                                    |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          | 🟢 No loss |          |                                                                                                                    |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             | 🟢 No loss |          |                                                                                                                    |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                | 🟢 No loss |          |                                                                                                                    |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       | 🟢 No loss |          | Encoded using special function                                                                                     |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         | 🟢 No loss |          |                                                                                                                    |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   | 🟢 No loss |          |                                                                                                                    |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          | 🟢 No loss |          |                                                                                                                    |
| **Style**                                                                                    |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  | 🟢 No loss |          | Encoded as [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div)                                |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | 🟢 No loss |          | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)                              |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        | 🟢 No loss |          |                                                                                                                    |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      | 🟢 No loss |          |                                                                                                                    |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    | 🟢 No loss |          |                                                                                                                    |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  | 🟢 No loss |          |                                                                                                                    |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          | 🟢 No loss |          |                                                                                                                    |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        | 🟢 No loss |          |                                                                                                                    |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             | 🟢 No loss |          |                                                                                                                    |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               | 🟢 No loss |          |                                                                                                                    |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                | 🟢 No loss |          |                                                                                                                    |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   | 🟢 No loss |          |                                                                                                                    |
| [Excerpt](https://stencila.ghost.io/docs/reference/schema/excerpt)                           | 🟢 No loss |          |                                                                                                                    |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               | 🟢 No loss |          |                                                                                                                    |
| [Island](https://stencila.ghost.io/docs/reference/schema/island)                             | 🟢 No loss |          |                                                                                                                    |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          | 🟢 No loss |          |                                                                                                                    |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              | 🟢 No loss |          |                                                                                                                    |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | 🟢 No loss |          |                                                                                                                    |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             | 🟢 No loss |          |                                                                                                                    |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | 🟢 No loss |          |                                                                                                                    |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | 🟢 No loss |          |                                                                                                                    |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              | 🟢 No loss |          |                                                                                                                    |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          | 🟢 No loss |          |                                                                                                                    |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        | 🟢 No loss |          |                                                                                                                    |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               | 🟢 No loss |          |                                                                                                                    |

See the Rust crate [`codec-dom`](https://github.com/stencila/stencila/tree/main/rust/codec-dom) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
