---
config:
  publish:
    ghost:
      slug: markdown-format
      state: publish
      tags:
      - '#doc'
      - Formats
      type: post
description: Markdown lightweight markup
title: Markdown
---
## Introduction

**File Extension:** `.smd`, `.md`, `.myst`, `.qmd` - Used when converting or exporting Stencila documents to various flavours of Markdown.

In Stencila we support various 'flavors' of Markdown with varying extensions added to Markdown for other tools to support. To get the most out of Stencila we recommend using `.smd` or Stencila Markdown, but we also have support for vanilla Markdown (`.md`), MyST Markdown (`.myst`) and Quarto Markdown ('.qmd`).

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

# Support

Stencila supports these operations for Markdown:

- decoding from a file
- decoding from a string
- encoding to a file
- encoding to a string

Support and degree of loss by node type:

| Node type                                                                                    | Encoding     | Decoding   | Notes                              |
| -------------------------------------------------------------------------------------------- | ------------ | ---------- | ---------------------------------- |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    | ⚠️ High loss |            |                                    |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 | ⚠️ High loss |            | Encoded using implemented function |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  | ⚠️ High loss |            | Encoded using implemented function |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       | ⚠️ High loss |            | Encoded using implemented function |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | ⚠️ High loss |            | Encoded using implemented function |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     | ⚠️ High loss |            |                                    |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           | ⚠️ High loss |            |                                    |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                | ⚠️ High loss |            |                                    |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       | ⚠️ High loss |            |                                    |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | ⚠️ High loss |            | Encoded using implemented function |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 | ⚠️ High loss |            |                                    |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     | ⚠️ High loss |            |                                    |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             | ⚠️ High loss |            | Encoded using implemented function |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        | ⚠️ High loss |            |                                    |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      | ⚠️ High loss |            |                                    |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             | ⚠️ High loss |            |                                    |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  | ⚠️ High loss |            |                                    |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   | ⚠️ High loss |            |                                    |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      | 🔷 Low loss   | 🔷 Low loss |                                    |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        | 🔷 Low loss   | 🔷 Low loss |                                    |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | ⚠️ High loss |            | Encoded using implemented function |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     | ⚠️ High loss |            | Encoded as `=={{content}}==`       |
| [Cite](https://stencila.ghost.io/docs/reference/schema/cite)                                 | ⚠️ High loss |            | Encoded using implemented function |
| [CiteGroup](https://stencila.ghost.io/docs/reference/schema/cite_group)                      | ⚠️ High loss |            |                                    |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  | ⚠️ High loss |            |                                    |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | 🟢 No loss    | 🟢 No loss  | Encoded as `_{{content}}_`         |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | 🟢 No loss    | 🟢 No loss  | Encoded as `{{content}}\n\n`       |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | ⚠️ High loss |            | Encoded as `<q>{{content}}</q>`    |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | ⚠️ High loss |            | Encoded as `~~{{content}}~~`       |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | 🟢 No loss    | 🟢 No loss  | Encoded as `**{{content}}**`       |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | 🟢 No loss    | 🟢 No loss  | Encoded as `~{{content}}~`         |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | 🟢 No loss    | 🟢 No loss  | Encoded as `^{{content}}^`         |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | 🟢 No loss    | 🟢 No loss  | Encoded as `***\n\n`               |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | 🟢 No loss    | 🟢 No loss  | Encoded as `<u>{{content}}</u>`    |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    | ⚠️ High loss |            |                                    |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        | ⚠️ High loss |            |                                    |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               | ⚠️ High loss |            |                                    |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      | ⚠️ High loss |            |                                    |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            | ⚠️ High loss |            | Encoded using implemented function |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | 🔷 Low loss   | 🔷 Low loss |                                    |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        | ⚠️ High loss |            | Encoded using implemented function |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      | ⚠️ High loss |            | Encoded using implemented function |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | 🟢 No loss    | 🟢 No loss  |                                    |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       | ⚠️ High loss |            | Encoded using implemented function |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          | ⚠️ High loss |            |                                    |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) | ⚠️ High loss |            |                                    |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              | ⚠️ High loss |            |                                    |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | ⚠️ High loss |            |                                    |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | ⚠️ High loss |            |                                    |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     | ⚠️ High loss |            | Encoded using implemented function |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              | ⚠️ High loss |            | Encoded using implemented function |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | ⚠️ High loss |            |                                    |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      | ⚠️ High loss |            | Encoded using implemented function |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              | ⚠️ High loss |            | Encoded using implemented function |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | 🔷 Low loss   | 🔷 Low loss |                                    |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        | ⚠️ High loss |            | Encoded using implemented function |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | 🔷 Low loss   | 🔷 Low loss |                                    |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | 🔷 Low loss   | 🔷 Low loss |                                    |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          | ⚠️ High loss |            | Encoded using implemented function |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             | ⚠️ High loss |            |                                    |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    | ⚠️ High loss |            |                                    |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | 🟢 No loss    | 🟢 No loss  |                                    |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    | ⚠️ High loss |            |                                    |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          | ⚠️ High loss |            | Encoded using implemented function |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | ⚠️ High loss |            |                                    |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              | ⚠️ High loss |            | Encoded using implemented function |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | ⚠️ High loss |            |                                    |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    | ⚠️ High loss |            | Encoded using implemented function |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            | ⚠️ High loss |            | Encoded using implemented function |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           | ⚠️ High loss |            |                                    |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | 🔷 Low loss   | 🔷 Low loss |                                    |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             | ⚠️ High loss |            |                                    |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                | ⚠️ High loss |            | Encoded using implemented function |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      | ⚠️ High loss |            | Encoded using implemented function |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                | ⚠️ High loss |            |                                    |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      | ⚠️ High loss |            |                                    |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    | ⚠️ High loss |            |                                    |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  | ⚠️ High loss |            |                                    |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                | ⚠️ High loss |            |                                    |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        | ⚠️ High loss |            | Encoded using implemented function |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 | ⚠️ High loss |            |                                    |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         | ⚠️ High loss |            |                                    |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          | ⚠️ High loss |            | Encoded using implemented function |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             | ⚠️ High loss |            | Encoded using implemented function |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                | ⚠️ High loss |            | Encoded using implemented function |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       | 🔷 Low loss   | 🔷 Low loss | Encoded using implemented function |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         | ⚠️ High loss |            |                                    |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   | ⚠️ High loss |            | Encoded using implemented function |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          | ⚠️ High loss |            | Encoded using implemented function |
| **Style**                                                                                    |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  | 🟢 No loss    | 🟢 No loss  | Encoded using implemented function |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | ⚠️ High loss |            | Encoded using implemented function |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        | ⚠️ High loss |            | Encoded using implemented function |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      | ⚠️ High loss |            | Encoded using implemented function |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    | ⚠️ High loss |            | Encoded using implemented function |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  | ⚠️ High loss |            | Encoded using implemented function |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          | ⚠️ High loss |            | Encoded using implemented function |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        | ⚠️ High loss |            | Encoded using implemented function |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             | ⚠️ High loss |            |                                    |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               | ⚠️ High loss |            |                                    |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                | ⚠️ High loss |            |                                    |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   | ⚠️ High loss |            |                                    |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               | ⚠️ High loss |            |                                    |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          | ⚠️ High loss |            | Encoded using implemented function |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              | ⚠️ High loss |            |                                    |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | ⚠️ High loss |            |                                    |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             | ⚠️ High loss |            |                                    |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | ⚠️ High loss |            |                                    |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | ⚠️ High loss |            |                                    |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              | ⚠️ High loss |            |                                    |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          | ⚠️ High loss |            |                                    |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        | ⚠️ High loss |            | Encoded using implemented function |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               | ⚠️ High loss |            |                                    |

See the Rust crate [`codec-markdown`](https://github.com/stencila/stencila/tree/main/rust/codec-markdown) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
