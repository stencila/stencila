---
title: JATS
description: Journal Article Tag Suite
config:
  publish:
    ghost:
      slug: jats
      tags:
        - "#docs"
        - Formats
---

# Introduction

The [JATS (Journal Article Tag Suite)](https://jats.nlm.nih.gov/) is an XML format that can be used to tag and describe scientific articles. It was developed by the NIH and has been adopted by several journals as a de facto standard for describing journal articles.

# Usage

Use the `.jats.xml` file extension, or the `--to jats` or `--from jats` options, when converting to/from JATS e.g.

```sh
stencila convert doc.smd doc.jats.xml
```

By default, the encoded JATS is un-indented. Use the `--pretty` option for indented XML but note that this may affect whitespace.

# Implementation

Stencila supports bi-direction conversion between Stencila document and JATS. Parsing of JATS built on top of the [`quick-xml`](https://crates.io/crates/quick-xml) Rust crate.

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

# Support

Stencila supports these operations for JATS:

- decoding from a file
- decoding from a string
- encoding to a file
- encoding to a string

Support and degree of loss by node type:

| Node type                                                                                    | Encoding   | Decoding   | Notes                                                                                                                                         |
| -------------------------------------------------------------------------------------------- | ---------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | 🔷 Low loss | 🔷 Low loss | Encoded as [`<article>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/article.html) using special function               |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | 🔷 Low loss | 🔷 Low loss | Encoded as [`<inline-media>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-media.html) using special function     |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    |            |            |                                                                                                                                               |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 |            |            |                                                                                                                                               |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  |            |            |                                                                                                                                               |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       |            |            |                                                                                                                                               |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | 🔷 Low loss |            | Encoded as [`<statement>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/statement.html)                                  |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     |            |            |                                                                                                                                               |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           |            |            |                                                                                                                                               |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                |            |            |                                                                                                                                               |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       |            |            |                                                                                                                                               |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | 🔷 Low loss |            | Encoded as [`<fig>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/fig.html)                                              |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 |            |            |                                                                                                                                               |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | 🔷 Low loss | 🔷 Low loss | Encoded as [`<inline-graphic>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-graphic.html) using special function |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  |            |            | Encoded as [`<inline-media>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-media.html) using special function     |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     |            |            |                                                                                                                                               |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             |            |            |                                                                                                                                               |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        |            |            |                                                                                                                                               |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      |            |            |                                                                                                                                               |
| [Reference](https://stencila.ghost.io/docs/reference/schema/reference)                       |            |            |                                                                                                                                               |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             |            |            |                                                                                                                                               |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  |            |            |                                                                                                                                               |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   |            |            |                                                                                                                                               |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               |            |            | Encoded using special function                                                                                                                |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      |            |            | Encoded using special function                                                                                                                |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        |            |            | Encoded using special function                                                                                                                |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | 🔷 Low loss | 🔷 Low loss | Encoded as [`<inline-media>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-media.html) using special function     |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | 🟢 No loss  | 🟢 No loss  | Encoded as [`<boxed-text>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/boxed-text.html)                                |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     |            |            | Encoded as [`<annotation>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/annotation.html)                                |
| [AppendixBreak](https://stencila.ghost.io/docs/reference/schema/appendix_break)              |            |            |                                                                                                                                               |
| [Citation](https://stencila.ghost.io/docs/reference/schema/citation)                         |            |            | Encoded using special function                                                                                                                |
| [CitationGroup](https://stencila.ghost.io/docs/reference/schema/citation_group)              |            |            | Encoded using special function                                                                                                                |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  |            |            |                                                                                                                                               |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | 🟢 No loss  | 🟢 No loss  | Encoded as [`<italic>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/italic.html)                                        |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | 🟢 No loss  | 🟢 No loss  | Encoded as [`<title>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/title.html) using special function                   |
| [InlinesBlock](https://stencila.ghost.io/docs/reference/schema/inlines_block)                |            |            |                                                                                                                                               |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | 🔷 Low loss | 🔷 Low loss | Encoded as [`<ext-link>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/ext-link.html)                                    |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | 🔷 Low loss |            | Encoded as [`<list>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list.html)                                            |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | 🔷 Low loss |            | Encoded as [`<list-item>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/list-item.html)                                  |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | 🟢 No loss  | 🟢 No loss  | Encoded as [`<fn>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/fn.html)                                                |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | 🟢 No loss  | 🟢 No loss  | Encoded as [`<p>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/p.html)                                                  |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | 🟢 No loss  | 🟢 No loss  | Encoded as [`<disp-quote>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/disp-quote.html)                                |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | 🟢 No loss  | 🟢 No loss  | Encoded as [`<inline-quote>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-quote.html)                            |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | 🟢 No loss  | 🟢 No loss  | Encoded as [`<sec>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/sec.html)                                              |
| [Sentence](https://stencila.ghost.io/docs/reference/schema/sentence)                         |            |            |                                                                                                                                               |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | 🟢 No loss  | 🟢 No loss  | Encoded as [`<strike>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/strike.html)                                        |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | 🟢 No loss  | 🟢 No loss  | Encoded as [`<bold>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/bold.html)                                            |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | 🟢 No loss  | 🟢 No loss  | Encoded as [`<sub>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/sub.html)                                              |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | 🟢 No loss  | 🟢 No loss  | Encoded as [`<sup>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/sup.html)                                              |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | 🟢 No loss  | 🟢 No loss  | Encoded using special function                                                                                                                |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | 🟢 No loss  | 🟢 No loss  | Encoded as [`<hr>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/hr.html)                                                |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | 🟢 No loss  | 🟢 No loss  | Encoded as [`<underline>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/underline.html)                                  |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | 🟢 No loss  | 🔷 Low loss | Encoded as [`<disp-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/disp-formula.html) using special function     |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | 🟢 No loss  | 🔷 Low loss | Encoded as [`<inline-formula>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/inline-formula.html) using special function |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | 🟢 No loss  |            | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html)                                            |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | 🔷 Low loss |            | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html)                                            |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | 🔷 Low loss | 🔷 Low loss | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html)                                            |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | 🟢 No loss  | 🟢 No loss  | Encoded as [`<code>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/code.html)                                            |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    |            |            |                                                                                                                                               |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        |            |            |                                                                                                                                               |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               |            |            |                                                                                                                                               |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      |            |            |                                                                                                                                               |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            |            |            |                                                                                                                                               |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | 🔷 Low loss |            |                                                                                                                                               |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        |            |            |                                                                                                                                               |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      |            |            |                                                                                                                                               |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | 🟢 No loss  | 🟢 No loss  |                                                                                                                                               |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       |            |            |                                                                                                                                               |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          |            |            |                                                                                                                                               |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) |            |            |                                                                                                                                               |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              |            |            |                                                                                                                                               |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | 🟢 No loss  | 🟢 No loss  | Encoded as [`<date>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/date.html) using special function                     |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | 🟢 No loss  | 🟢 No loss  | Encoded as [`<date-time>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/date-time.html) using special function           |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     |            |            |                                                                                                                                               |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              |            |            |                                                                                                                                               |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | 🟢 No loss  | 🟢 No loss  | Encoded as [`<duration>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/duration.html) using special function             |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      |            |            |                                                                                                                                               |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              |            |            |                                                                                                                                               |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | 🔷 Low loss |            |                                                                                                                                               |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        |            |            |                                                                                                                                               |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | 🔷 Low loss |            |                                                                                                                                               |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | 🔷 Low loss |            |                                                                                                                                               |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          |            |            |                                                                                                                                               |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             |            |            |                                                                                                                                               |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    |            |            |                                                                                                                                               |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | 🟢 No loss  | 🟢 No loss  |                                                                                                                                               |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    |            |            |                                                                                                                                               |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          |            |            |                                                                                                                                               |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | 🟢 No loss  | 🟢 No loss  | Encoded as [`<time>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/time.html) using special function                     |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              |            |            |                                                                                                                                               |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | 🟢 No loss  | 🟢 No loss  | Encoded as [`<timestamp>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/timestamp.html) using special function           |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    |            |            |                                                                                                                                               |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            |            |            |                                                                                                                                               |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           |            |            |                                                                                                                                               |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | 🔷 Low loss |            |                                                                                                                                               |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             |            |            |                                                                                                                                               |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                |            |            |                                                                                                                                               |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      |            |            |                                                                                                                                               |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                |            |            |                                                                                                                                               |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      |            |            |                                                                                                                                               |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    |            |            |                                                                                                                                               |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  |            |            |                                                                                                                                               |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                |            |            |                                                                                                                                               |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        |            |            |                                                                                                                                               |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 |            |            |                                                                                                                                               |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         |            |            |                                                                                                                                               |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          |            |            |                                                                                                                                               |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             |            |            |                                                                                                                                               |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                |            |            |                                                                                                                                               |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       |            |            | Encoded as [`<parameter>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/parameter.html)                                  |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         |            |            |                                                                                                                                               |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   |            |            |                                                                                                                                               |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          |            |            |                                                                                                                                               |
| **Style**                                                                                    |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  |            |            |                                                                                                                                               |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | 🟢 No loss  | 🟢 No loss  | Encoded as [`<styled-content>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/styled-content.html)                        |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        |            |            |                                                                                                                                               |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      |            |            |                                                                                                                                               |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    |            |            |                                                                                                                                               |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  |            |            |                                                                                                                                               |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          |            |            |                                                                                                                                               |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        |            |            |                                                                                                                                               |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             |            |            |                                                                                                                                               |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               |            |            |                                                                                                                                               |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                |            |            |                                                                                                                                               |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   |            |            |                                                                                                                                               |
| [Excerpt](https://stencila.ghost.io/docs/reference/schema/excerpt)                           |            |            |                                                                                                                                               |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               |            |            |                                                                                                                                               |
| [Island](https://stencila.ghost.io/docs/reference/schema/island)                             |            |            |                                                                                                                                               |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          |            |            |                                                                                                                                               |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              |            |            |                                                                                                                                               |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | 🔷 Low loss |            | Encoded as [`<institution>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/institution.html)                              |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             |            |            |                                                                                                                                               |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | 🔷 Low loss |            | Encoded as [`<address>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/address.html)                                      |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | 🔷 Low loss |            | Encoded as [`<product>`](https://jats.nlm.nih.gov/articleauthoring/tag-library/1.3/element/product.html)                                      |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              |            |            |                                                                                                                                               |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          |            |            |                                                                                                                                               |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        |            |            |                                                                                                                                               |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               |            |            |                                                                                                                                               |

See the Rust crate [`codec-jats`](https://github.com/stencila/stencila/tree/main/rust/codec-jats) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
