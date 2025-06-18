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

| Node type                                                                                    | Encoding   | Decoding | Notes                                                                                                              |
| -------------------------------------------------------------------------------------------- | ---------- | -------- | ------------------------------------------------------------------------------------------------------------------ |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | 🔷 Low loss |          | Encoded as [`<article>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/article)                        |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | 🔷 Low loss |          | Encoded as [`<audio>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/audio) using special function     |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    | 🔷 Low loss |          |                                                                                                                    |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 | 🔷 Low loss |          |                                                                                                                    |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  | 🔷 Low loss |          |                                                                                                                    |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       | 🔷 Low loss |          |                                                                                                                    |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | 🔷 Low loss |          |                                                                                                                    |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     | 🔷 Low loss |          |                                                                                                                    |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           | 🔷 Low loss |          |                                                                                                                    |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                | 🔷 Low loss |          |                                                                                                                    |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       | 🔷 Low loss |          |                                                                                                                    |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | 🔷 Low loss |          | Encoded as [`<figure>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/figure)                          |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 | 🔷 Low loss |          |                                                                                                                    |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | 🔷 Low loss |          | Encoded as [`<img>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/img) using special function         |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  | 🔷 Low loss |          |                                                                                                                    |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     | 🔷 Low loss |          |                                                                                                                    |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             | 🔷 Low loss |          |                                                                                                                    |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        | 🔷 Low loss |          |                                                                                                                    |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      | 🔷 Low loss |          |                                                                                                                    |
| [Reference](https://stencila.ghost.io/docs/reference/schema/reference)                       | 🔷 Low loss |          |                                                                                                                    |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             | 🔷 Low loss |          |                                                                                                                    |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  | 🔷 Low loss |          |                                                                                                                    |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   | 🔷 Low loss |          |                                                                                                                    |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               | 🔷 Low loss |          | Encoded using special function                                                                                     |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      | 🔷 Low loss |          | Encoded as [`<td>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/td)                                  |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        | 🔷 Low loss |          | Encoded as [`<tr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/tr)                                  |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | 🔷 Low loss |          | Encoded as [`<video>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video) using special function     |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | 🔷 Low loss |          | Encoded as [`<aside>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/aside)                            |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     | 🔷 Low loss |          |                                                                                                                    |
| [Citation](https://stencila.ghost.io/docs/reference/schema/citation)                         | 🔷 Low loss |          |                                                                                                                    |
| [CitationGroup](https://stencila.ghost.io/docs/reference/schema/citation_group)              | 🔷 Low loss |          |                                                                                                                    |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  | 🔷 Low loss |          |                                                                                                                    |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | 🟢 No loss  |          | Encoded as [`<em>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/em)                                  |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | 🟢 No loss  |          | Encoded using special function                                                                                     |
| [InlinesBlock](https://stencila.ghost.io/docs/reference/schema/inlines_block)                | 🔷 Low loss |          |                                                                                                                    |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | 🔷 Low loss |          | Encoded as [`<a>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/a)                                    |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | 🔷 Low loss |          | Encoded using special function                                                                                     |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | 🔷 Low loss |          | Encoded as [`<li>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/li)                                  |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | 🔷 Low loss |          |                                                                                                                    |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | 🟢 No loss  |          | Encoded as [`<p>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/p)                                    |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | 🔷 Low loss |          | Encoded as [`<blockquote>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/blockquote)                  |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | 🔷 Low loss |          | Encoded as [`<q>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/q)                                    |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | 🟢 No loss  |          | Encoded as [`<section>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/section) using special function |
| [Sentence](https://stencila.ghost.io/docs/reference/schema/sentence)                         | 🔷 Low loss |          |                                                                                                                    |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | 🔷 Low loss |          | Encoded as [`<s>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/s)                                    |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | 🟢 No loss  |          | Encoded as [`<strong>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/strong)                          |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | 🟢 No loss  |          | Encoded as [`<sub>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sub)                                |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | 🟢 No loss  |          | Encoded as [`<sup>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/sup)                                |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | 🟢 No loss  |          | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)                              |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | 🟢 No loss  |          | Encoded as [`<hr>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/hr)                                  |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | 🟢 No loss  |          | Encoded as [`<u>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/u)                                    |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | 🔷 Low loss |          | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                              |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | 🔷 Low loss |          | Encoded as [`<math>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/math)                              |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | 🟢 No loss  |          | Encoded as [`<pre>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/pre)                                |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | 🔷 Low loss |          |                                                                                                                    |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | 🔷 Low loss |          |                                                                                                                    |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | 🟢 No loss  |          | Encoded as [`<code>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/code)                              |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    | 🔷 Low loss |          |                                                                                                                    |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        | 🔷 Low loss |          |                                                                                                                    |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               | 🔷 Low loss |          |                                                                                                                    |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      | 🔷 Low loss |          |                                                                                                                    |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            | 🔷 Low loss |          |                                                                                                                    |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | 🔷 Low loss |          |                                                                                                                    |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        | 🔷 Low loss |          |                                                                                                                    |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      | 🔷 Low loss |          |                                                                                                                    |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | 🟢 No loss  |          |                                                                                                                    |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       | 🔷 Low loss |          |                                                                                                                    |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          | 🔷 Low loss |          |                                                                                                                    |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) | 🔷 Low loss |          |                                                                                                                    |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              | 🔷 Low loss |          |                                                                                                                    |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | 🔷 Low loss |          |                                                                                                                    |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | 🔷 Low loss |          |                                                                                                                    |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     | 🔷 Low loss |          |                                                                                                                    |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              | 🔷 Low loss |          |                                                                                                                    |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | 🔷 Low loss |          |                                                                                                                    |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      | 🔷 Low loss |          |                                                                                                                    |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              | 🔷 Low loss |          |                                                                                                                    |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | 🔷 Low loss |          |                                                                                                                    |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        | 🔷 Low loss |          |                                                                                                                    |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | 🔷 Low loss |          |                                                                                                                    |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | 🔷 Low loss |          |                                                                                                                    |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          | 🔷 Low loss |          |                                                                                                                    |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             | 🔷 Low loss |          |                                                                                                                    |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    | 🔷 Low loss |          |                                                                                                                    |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | 🟢 No loss  |          |                                                                                                                    |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    | 🔷 Low loss |          |                                                                                                                    |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          | 🔷 Low loss |          |                                                                                                                    |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | 🔷 Low loss |          |                                                                                                                    |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              | 🔷 Low loss |          |                                                                                                                    |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | 🔷 Low loss |          |                                                                                                                    |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    | 🔷 Low loss |          |                                                                                                                    |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            | 🔷 Low loss |          |                                                                                                                    |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           | 🔷 Low loss |          |                                                                                                                    |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | 🔷 Low loss |          |                                                                                                                    |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             | 🔷 Low loss |          | Encoded as [`<button>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button)                          |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                | 🔷 Low loss |          |                                                                                                                    |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      | 🔷 Low loss |          |                                                                                                                    |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                | 🔷 Low loss |          |                                                                                                                    |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      | 🔷 Low loss |          |                                                                                                                    |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    | 🔷 Low loss |          |                                                                                                                    |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  | 🔷 Low loss |          |                                                                                                                    |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                | 🔷 Low loss |          |                                                                                                                    |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        | 🔷 Low loss |          |                                                                                                                    |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 | 🔷 Low loss |          |                                                                                                                    |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         | 🔷 Low loss |          |                                                                                                                    |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          | 🔷 Low loss |          |                                                                                                                    |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             | 🔷 Low loss |          |                                                                                                                    |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                | 🔷 Low loss |          |                                                                                                                    |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       | 🔷 Low loss |          | Encoded using special function                                                                                     |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         | 🔷 Low loss |          |                                                                                                                    |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   | 🔷 Low loss |          |                                                                                                                    |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          | 🔷 Low loss |          |                                                                                                                    |
| **Style**                                                                                    |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  | 🔷 Low loss |          | Encoded as [`<div>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/div)                                |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | 🔷 Low loss |          | Encoded as [`<span>`](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/span)                              |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        | 🔷 Low loss |          |                                                                                                                    |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      | 🔷 Low loss |          |                                                                                                                    |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    | 🔷 Low loss |          |                                                                                                                    |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  | 🔷 Low loss |          |                                                                                                                    |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          | 🔷 Low loss |          |                                                                                                                    |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        | 🔷 Low loss |          |                                                                                                                    |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             | 🔷 Low loss |          |                                                                                                                    |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               | 🔷 Low loss |          |                                                                                                                    |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                | 🔷 Low loss |          |                                                                                                                    |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   | 🔷 Low loss |          |                                                                                                                    |
| [Excerpt](https://stencila.ghost.io/docs/reference/schema/excerpt)                           | 🔷 Low loss |          |                                                                                                                    |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               | 🔷 Low loss |          |                                                                                                                    |
| [Island](https://stencila.ghost.io/docs/reference/schema/island)                             | 🔷 Low loss |          |                                                                                                                    |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          | 🔷 Low loss |          |                                                                                                                    |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              | 🔷 Low loss |          |                                                                                                                    |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | 🔷 Low loss |          |                                                                                                                    |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             | 🔷 Low loss |          |                                                                                                                    |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | 🔷 Low loss |          |                                                                                                                    |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | 🔷 Low loss |          |                                                                                                                    |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              | 🔷 Low loss |          |                                                                                                                    |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          | 🔷 Low loss |          |                                                                                                                    |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        | 🔷 Low loss |          |                                                                                                                    |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               | 🔷 Low loss |          |                                                                                                                    |

See the Rust crate [`codec-html`](https://github.com/stencila/stencila/tree/main/rust/codec-html) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
