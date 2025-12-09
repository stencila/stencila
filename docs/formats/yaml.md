---
title: YAML
description: A human-readable data serialization format
config:
  publish:
    ghost:
      slug: yaml
      tags:
        - "#docs"
        - Formats
---

# Introduction

[YAML (YAML Ain't Markup Language)](https://yaml.org/) is a human-readable data serialization format commonly used for configuration files and data representation. It is known for its simplicity and readability, making it a preferred choice for settings and data structures. YAML's structure is based on indentation, allowing users to represent data hierarchies in an easily understandable manner.

Stencila provides support for YAML as a more human-readable, while still lossless, alternative to [JSON](../json) for storing documents.

# Usage

Use the `.yaml` file extension, or the `--to yaml` or `--from yaml` options, when converting to/from YAML e.g.

```sh
stencila convert doc.smd doc.yaml
```

# Implementation

Stencila support lossless, bi-directional conversion between Stencila documents and YAML. The `codec-yaml` Rust crate implements `from_yaml` and `to_yaml` methods for all node types in Stencila Schema, powered by [`serde_yaml`](https://crates.io/crates/serde_yaml).

When the `--standalone` option is used (the default for encoding to files), two properties are added to the YAML encoding of root nodes to improve interoperability:

- a `$schema` property which links to the [JSON Schema](https://json-schema.org) for the node type
- a `@context` property which links to the [JSON-LD](https://json-ld.org) context for the Stencila Schema

For example,

```yaml
$schema: https://stencila.org/Article.schema.json
"@context": https://stencila.org/context.jsonld
type: Article
```

<!-- prettier-ignore-start -->
<!-- CODEC-DOCS:START -->

# Support

Stencila supports these operations for YAML:

- decoding from a file
- decoding from a string
- encoding to a file
- encoding to a string

Support and degree of loss by node type:

| Node type                                                                                    | Encoding  | Decoding  | Notes |
| -------------------------------------------------------------------------------------------- | --------- | --------- | ----- |
| **Works**                                                                                    |
| [Article](https://stencila.ghost.io/docs/reference/schema/article)                           | 🟢 No loss | 🟢 No loss |       |
| [AudioObject](https://stencila.ghost.io/docs/reference/schema/audio_object)                  | 🟢 No loss | 🟢 No loss |       |
| [AuthorRole](https://stencila.ghost.io/docs/reference/schema/author_role)                    | 🟢 No loss | 🟢 No loss |       |
| [Bibliography](https://stencila.ghost.io/docs/reference/schema/bibliography)                 | 🟢 No loss | 🟢 No loss |       |
| [Chat](https://stencila.ghost.io/docs/reference/schema/chat)                                 | 🟢 No loss | 🟢 No loss |       |
| [ChatMessage](https://stencila.ghost.io/docs/reference/schema/chat_message)                  | 🟢 No loss | 🟢 No loss |       |
| [ChatMessageGroup](https://stencila.ghost.io/docs/reference/schema/chat_message_group)       | 🟢 No loss | 🟢 No loss |       |
| [Claim](https://stencila.ghost.io/docs/reference/schema/claim)                               | 🟢 No loss | 🟢 No loss |       |
| [Collection](https://stencila.ghost.io/docs/reference/schema/collection)                     | 🟢 No loss | 🟢 No loss |       |
| [Comment](https://stencila.ghost.io/docs/reference/schema/comment)                           | 🟢 No loss | 🟢 No loss |       |
| [CreativeWork](https://stencila.ghost.io/docs/reference/schema/creative_work)                | 🟢 No loss | 🟢 No loss |       |
| [Directory](https://stencila.ghost.io/docs/reference/schema/directory)                       | 🟢 No loss | 🟢 No loss |       |
| [Figure](https://stencila.ghost.io/docs/reference/schema/figure)                             | 🟢 No loss | 🟢 No loss |       |
| [File](https://stencila.ghost.io/docs/reference/schema/file)                                 | 🟢 No loss | 🟢 No loss |       |
| [ImageObject](https://stencila.ghost.io/docs/reference/schema/image_object)                  | 🟢 No loss | 🟢 No loss |       |
| [MediaObject](https://stencila.ghost.io/docs/reference/schema/media_object)                  | 🟢 No loss | 🟢 No loss |       |
| [Periodical](https://stencila.ghost.io/docs/reference/schema/periodical)                     | 🟢 No loss | 🟢 No loss |       |
| [Prompt](https://stencila.ghost.io/docs/reference/schema/prompt)                             | 🟢 No loss | 🟢 No loss |       |
| [PublicationIssue](https://stencila.ghost.io/docs/reference/schema/publication_issue)        | 🟢 No loss | 🟢 No loss |       |
| [PublicationVolume](https://stencila.ghost.io/docs/reference/schema/publication_volume)      | 🟢 No loss | 🟢 No loss |       |
| [Reference](https://stencila.ghost.io/docs/reference/schema/reference)                       | 🟢 No loss | 🟢 No loss |       |
| [Review](https://stencila.ghost.io/docs/reference/schema/review)                             | 🟢 No loss | 🟢 No loss |       |
| [SoftwareApplication](https://stencila.ghost.io/docs/reference/schema/software_application)  | 🟢 No loss | 🟢 No loss |       |
| [SoftwareSourceCode](https://stencila.ghost.io/docs/reference/schema/software_source_code)   | 🟢 No loss | 🟢 No loss |       |
| [Supplement](https://stencila.ghost.io/docs/reference/schema/supplement)                     | 🟢 No loss | 🟢 No loss |       |
| [Table](https://stencila.ghost.io/docs/reference/schema/table)                               | 🟢 No loss | 🟢 No loss |       |
| [TableCell](https://stencila.ghost.io/docs/reference/schema/table_cell)                      | 🟢 No loss | 🟢 No loss |       |
| [TableRow](https://stencila.ghost.io/docs/reference/schema/table_row)                        | 🟢 No loss | 🟢 No loss |       |
| [VideoObject](https://stencila.ghost.io/docs/reference/schema/video_object)                  | 🟢 No loss | 🟢 No loss |       |
| **Prose**                                                                                    |
| [Admonition](https://stencila.ghost.io/docs/reference/schema/admonition)                     | 🟢 No loss | 🟢 No loss |       |
| [Annotation](https://stencila.ghost.io/docs/reference/schema/annotation)                     | 🟢 No loss | 🟢 No loss |       |
| [AppendixBreak](https://stencila.ghost.io/docs/reference/schema/appendix_break)              | 🟢 No loss | 🟢 No loss |       |
| [Citation](https://stencila.ghost.io/docs/reference/schema/citation)                         | 🟢 No loss | 🟢 No loss |       |
| [CitationGroup](https://stencila.ghost.io/docs/reference/schema/citation_group)              | 🟢 No loss | 🟢 No loss |       |
| [DefinedTerm](https://stencila.ghost.io/docs/reference/schema/defined_term)                  | 🟢 No loss | 🟢 No loss |       |
| [Emphasis](https://stencila.ghost.io/docs/reference/schema/emphasis)                         | 🟢 No loss | 🟢 No loss |       |
| [Heading](https://stencila.ghost.io/docs/reference/schema/heading)                           | 🟢 No loss | 🟢 No loss |       |
| [InlinesBlock](https://stencila.ghost.io/docs/reference/schema/inlines_block)                | 🟢 No loss | 🟢 No loss |       |
| [Link](https://stencila.ghost.io/docs/reference/schema/link)                                 | 🟢 No loss | 🟢 No loss |       |
| [List](https://stencila.ghost.io/docs/reference/schema/list)                                 | 🟢 No loss | 🟢 No loss |       |
| [ListItem](https://stencila.ghost.io/docs/reference/schema/list_item)                        | 🟢 No loss | 🟢 No loss |       |
| [Note](https://stencila.ghost.io/docs/reference/schema/note)                                 | 🟢 No loss | 🟢 No loss |       |
| [Paragraph](https://stencila.ghost.io/docs/reference/schema/paragraph)                       | 🟢 No loss | 🟢 No loss |       |
| [QuoteBlock](https://stencila.ghost.io/docs/reference/schema/quote_block)                    | 🟢 No loss | 🟢 No loss |       |
| [QuoteInline](https://stencila.ghost.io/docs/reference/schema/quote_inline)                  | 🟢 No loss | 🟢 No loss |       |
| [Section](https://stencila.ghost.io/docs/reference/schema/section)                           | 🟢 No loss | 🟢 No loss |       |
| [Sentence](https://stencila.ghost.io/docs/reference/schema/sentence)                         | 🟢 No loss | 🟢 No loss |       |
| [Strikeout](https://stencila.ghost.io/docs/reference/schema/strikeout)                       | 🟢 No loss | 🟢 No loss |       |
| [Strong](https://stencila.ghost.io/docs/reference/schema/strong)                             | 🟢 No loss | 🟢 No loss |       |
| [Subscript](https://stencila.ghost.io/docs/reference/schema/subscript)                       | 🟢 No loss | 🟢 No loss |       |
| [Superscript](https://stencila.ghost.io/docs/reference/schema/superscript)                   | 🟢 No loss | 🟢 No loss |       |
| [Text](https://stencila.ghost.io/docs/reference/schema/text)                                 | 🟢 No loss | 🟢 No loss |       |
| [ThematicBreak](https://stencila.ghost.io/docs/reference/schema/thematic_break)              | 🟢 No loss | 🟢 No loss |       |
| [Underline](https://stencila.ghost.io/docs/reference/schema/underline)                       | 🟢 No loss | 🟢 No loss |       |
| **Math**                                                                                     |
| [MathBlock](https://stencila.ghost.io/docs/reference/schema/math_block)                      | 🟢 No loss | 🟢 No loss |       |
| [MathInline](https://stencila.ghost.io/docs/reference/schema/math_inline)                    | 🟢 No loss | 🟢 No loss |       |
| **Code**                                                                                     |
| [CodeBlock](https://stencila.ghost.io/docs/reference/schema/code_block)                      | 🟢 No loss | 🟢 No loss |       |
| [CodeChunk](https://stencila.ghost.io/docs/reference/schema/code_chunk)                      | 🟢 No loss | 🟢 No loss |       |
| [CodeExpression](https://stencila.ghost.io/docs/reference/schema/code_expression)            | 🟢 No loss | 🟢 No loss |       |
| [CodeInline](https://stencila.ghost.io/docs/reference/schema/code_inline)                    | 🟢 No loss | 🟢 No loss |       |
| [CompilationMessage](https://stencila.ghost.io/docs/reference/schema/compilation_message)    | 🟢 No loss | 🟢 No loss |       |
| [ExecutionMessage](https://stencila.ghost.io/docs/reference/schema/execution_message)        | 🟢 No loss | 🟢 No loss |       |
| **Data**                                                                                     |
| [Array](https://stencila.ghost.io/docs/reference/schema/array)                               | 🟢 No loss | 🟢 No loss |       |
| [ArrayHint](https://stencila.ghost.io/docs/reference/schema/array_hint)                      | 🟢 No loss | 🟢 No loss |       |
| [ArrayValidator](https://stencila.ghost.io/docs/reference/schema/array_validator)            | 🟢 No loss | 🟢 No loss |       |
| [Boolean](https://stencila.ghost.io/docs/reference/schema/boolean)                           | 🟢 No loss | 🟢 No loss |       |
| [BooleanValidator](https://stencila.ghost.io/docs/reference/schema/boolean_validator)        | 🟢 No loss | 🟢 No loss |       |
| [ConstantValidator](https://stencila.ghost.io/docs/reference/schema/constant_validator)      | 🟢 No loss | 🟢 No loss |       |
| [Cord](https://stencila.ghost.io/docs/reference/schema/cord)                                 | 🟢 No loss | 🟢 No loss |       |
| [Datatable](https://stencila.ghost.io/docs/reference/schema/datatable)                       | 🟢 No loss | 🟢 No loss |       |
| [DatatableColumn](https://stencila.ghost.io/docs/reference/schema/datatable_column)          | 🟢 No loss | 🟢 No loss |       |
| [DatatableColumnHint](https://stencila.ghost.io/docs/reference/schema/datatable_column_hint) | 🟢 No loss | 🟢 No loss |       |
| [DatatableHint](https://stencila.ghost.io/docs/reference/schema/datatable_hint)              | 🟢 No loss | 🟢 No loss |       |
| [Date](https://stencila.ghost.io/docs/reference/schema/date)                                 | 🟢 No loss | 🟢 No loss |       |
| [DateTime](https://stencila.ghost.io/docs/reference/schema/date_time)                        | 🟢 No loss | 🟢 No loss |       |
| [DateTimeValidator](https://stencila.ghost.io/docs/reference/schema/date_time_validator)     | 🟢 No loss | 🟢 No loss |       |
| [DateValidator](https://stencila.ghost.io/docs/reference/schema/date_validator)              | 🟢 No loss | 🟢 No loss |       |
| [Duration](https://stencila.ghost.io/docs/reference/schema/duration)                         | 🟢 No loss | 🟢 No loss |       |
| [DurationValidator](https://stencila.ghost.io/docs/reference/schema/duration_validator)      | 🟢 No loss | 🟢 No loss |       |
| [EnumValidator](https://stencila.ghost.io/docs/reference/schema/enum_validator)              | 🟢 No loss | 🟢 No loss |       |
| [Integer](https://stencila.ghost.io/docs/reference/schema/integer)                           | 🟢 No loss | 🟢 No loss |       |
| [IntegerValidator](https://stencila.ghost.io/docs/reference/schema/integer_validator)        | 🟢 No loss | 🟢 No loss |       |
| [Null](https://stencila.ghost.io/docs/reference/schema/null)                                 | 🟢 No loss | 🟢 No loss |       |
| [Number](https://stencila.ghost.io/docs/reference/schema/number)                             | 🟢 No loss | 🟢 No loss |       |
| [NumberValidator](https://stencila.ghost.io/docs/reference/schema/number_validator)          | 🟢 No loss | 🟢 No loss |       |
| [Object](https://stencila.ghost.io/docs/reference/schema/object)                             | 🟢 No loss | 🟢 No loss |       |
| [ObjectHint](https://stencila.ghost.io/docs/reference/schema/object_hint)                    | 🟢 No loss | 🟢 No loss |       |
| [String](https://stencila.ghost.io/docs/reference/schema/string)                             | 🟢 No loss | 🟢 No loss |       |
| [StringHint](https://stencila.ghost.io/docs/reference/schema/string_hint)                    | 🟢 No loss | 🟢 No loss |       |
| [StringValidator](https://stencila.ghost.io/docs/reference/schema/string_validator)          | 🟢 No loss | 🟢 No loss |       |
| [Time](https://stencila.ghost.io/docs/reference/schema/time)                                 | 🟢 No loss | 🟢 No loss |       |
| [TimeValidator](https://stencila.ghost.io/docs/reference/schema/time_validator)              | 🟢 No loss | 🟢 No loss |       |
| [Timestamp](https://stencila.ghost.io/docs/reference/schema/timestamp)                       | 🟢 No loss | 🟢 No loss |       |
| [TimestampValidator](https://stencila.ghost.io/docs/reference/schema/timestamp_validator)    | 🟢 No loss | 🟢 No loss |       |
| [TupleValidator](https://stencila.ghost.io/docs/reference/schema/tuple_validator)            | 🟢 No loss | 🟢 No loss |       |
| [Unknown](https://stencila.ghost.io/docs/reference/schema/unknown)                           | 🟢 No loss | 🟢 No loss |       |
| [UnsignedInteger](https://stencila.ghost.io/docs/reference/schema/unsigned_integer)          | 🟢 No loss | 🟢 No loss |       |
| **Flow**                                                                                     |
| [Button](https://stencila.ghost.io/docs/reference/schema/button)                             | 🟢 No loss | 🟢 No loss |       |
| [CallArgument](https://stencila.ghost.io/docs/reference/schema/call_argument)                | 🟢 No loss | 🟢 No loss |       |
| [CallBlock](https://stencila.ghost.io/docs/reference/schema/call_block)                      | 🟢 No loss | 🟢 No loss |       |
| [CodeLocation](https://stencila.ghost.io/docs/reference/schema/code_location)                | 🟢 No loss | 🟢 No loss |       |
| [CompilationDigest](https://stencila.ghost.io/docs/reference/schema/compilation_digest)      | 🟢 No loss | 🟢 No loss |       |
| [ExecutionDependant](https://stencila.ghost.io/docs/reference/schema/execution_dependant)    | 🟢 No loss | 🟢 No loss |       |
| [ExecutionDependency](https://stencila.ghost.io/docs/reference/schema/execution_dependency)  | 🟢 No loss | 🟢 No loss |       |
| [ExecutionTag](https://stencila.ghost.io/docs/reference/schema/execution_tag)                | 🟢 No loss | 🟢 No loss |       |
| [ForBlock](https://stencila.ghost.io/docs/reference/schema/for_block)                        | 🟢 No loss | 🟢 No loss |       |
| [Form](https://stencila.ghost.io/docs/reference/schema/form)                                 | 🟢 No loss | 🟢 No loss |       |
| [Function](https://stencila.ghost.io/docs/reference/schema/function)                         | 🟢 No loss | 🟢 No loss |       |
| [IfBlock](https://stencila.ghost.io/docs/reference/schema/if_block)                          | 🟢 No loss | 🟢 No loss |       |
| [IfBlockClause](https://stencila.ghost.io/docs/reference/schema/if_block_clause)             | 🟢 No loss | 🟢 No loss |       |
| [IncludeBlock](https://stencila.ghost.io/docs/reference/schema/include_block)                | 🟢 No loss | 🟢 No loss |       |
| [Parameter](https://stencila.ghost.io/docs/reference/schema/parameter)                       | 🟢 No loss | 🟢 No loss |       |
| [Variable](https://stencila.ghost.io/docs/reference/schema/variable)                         | 🟢 No loss | 🟢 No loss |       |
| [Walkthrough](https://stencila.ghost.io/docs/reference/schema/walkthrough)                   | 🟢 No loss | 🟢 No loss |       |
| [WalkthroughStep](https://stencila.ghost.io/docs/reference/schema/walkthrough_step)          | 🟢 No loss | 🟢 No loss |       |
| **Style**                                                                                    |
| [Page](https://stencila.ghost.io/docs/reference/schema/page)                                 | 🟢 No loss | 🟢 No loss |       |
| [StyledBlock](https://stencila.ghost.io/docs/reference/schema/styled_block)                  | 🟢 No loss | 🟢 No loss |       |
| [StyledInline](https://stencila.ghost.io/docs/reference/schema/styled_inline)                | 🟢 No loss | 🟢 No loss |       |
| **Edits**                                                                                    |
| [InstructionBlock](https://stencila.ghost.io/docs/reference/schema/instruction_block)        | 🟢 No loss | 🟢 No loss |       |
| [InstructionInline](https://stencila.ghost.io/docs/reference/schema/instruction_inline)      | 🟢 No loss | 🟢 No loss |       |
| [InstructionMessage](https://stencila.ghost.io/docs/reference/schema/instruction_message)    | 🟢 No loss | 🟢 No loss |       |
| [PromptBlock](https://stencila.ghost.io/docs/reference/schema/prompt_block)                  | 🟢 No loss | 🟢 No loss |       |
| [SuggestionBlock](https://stencila.ghost.io/docs/reference/schema/suggestion_block)          | 🟢 No loss | 🟢 No loss |       |
| [SuggestionInline](https://stencila.ghost.io/docs/reference/schema/suggestion_inline)        | 🟢 No loss | 🟢 No loss |       |
| **Config**                                                                                   |
| [Config](https://stencila.ghost.io/docs/reference/schema/config)                             | 🟢 No loss | 🟢 No loss |       |
| **Other**                                                                                    |
| [Brand](https://stencila.ghost.io/docs/reference/schema/brand)                               | 🟢 No loss | 🟢 No loss |       |
| [ContactPoint](https://stencila.ghost.io/docs/reference/schema/contact_point)                | 🟢 No loss | 🟢 No loss |       |
| [Enumeration](https://stencila.ghost.io/docs/reference/schema/enumeration)                   | 🟢 No loss | 🟢 No loss |       |
| [Excerpt](https://stencila.ghost.io/docs/reference/schema/excerpt)                           | 🟢 No loss | 🟢 No loss |       |
| [Grant](https://stencila.ghost.io/docs/reference/schema/grant)                               | 🟢 No loss | 🟢 No loss |       |
| [Island](https://stencila.ghost.io/docs/reference/schema/island)                             | 🟢 No loss | 🟢 No loss |       |
| [ModelParameters](https://stencila.ghost.io/docs/reference/schema/model_parameters)          | 🟢 No loss | 🟢 No loss |       |
| [MonetaryGrant](https://stencila.ghost.io/docs/reference/schema/monetary_grant)              | 🟢 No loss | 🟢 No loss |       |
| [Organization](https://stencila.ghost.io/docs/reference/schema/organization)                 | 🟢 No loss | 🟢 No loss |       |
| [Person](https://stencila.ghost.io/docs/reference/schema/person)                             | 🟢 No loss | 🟢 No loss |       |
| [PostalAddress](https://stencila.ghost.io/docs/reference/schema/postal_address)              | 🟢 No loss | 🟢 No loss |       |
| [Product](https://stencila.ghost.io/docs/reference/schema/product)                           | 🟢 No loss | 🟢 No loss |       |
| [PropertyValue](https://stencila.ghost.io/docs/reference/schema/property_value)              | 🟢 No loss | 🟢 No loss |       |
| [ProvenanceCount](https://stencila.ghost.io/docs/reference/schema/provenance_count)          | 🟢 No loss | 🟢 No loss |       |
| [RawBlock](https://stencila.ghost.io/docs/reference/schema/raw_block)                        | 🟢 No loss | 🟢 No loss |       |
| [Thing](https://stencila.ghost.io/docs/reference/schema/thing)                               | 🟢 No loss | 🟢 No loss |       |

See the Rust crate [`codec-yaml`](https://github.com/stencila/stencila/tree/main/rust/codec-yaml) for more details.


<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
