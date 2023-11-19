# Markdown

## Introduction

Markdown is a lightweight markup language widely used for formatting plain text documents. It provides a simple and human-readable way to structure text and add basic styling, such as headers, lists, links, and emphasis. Markdown's benefits include ease of use, and compatibility with various web and documentation platforms.

## Implementation

Stencila support bi-directional conversion between Stencila documents and Markdown. 

### Stencila to Markdown

Three internal Rust crates are involved in the conversion from Stencila documents to Markdown:

- The `codec-markdown` crate defines the `MarkdownCodec` `struct` which implements `to_string` method of the `Codec` `trait` by calling the `to_markdown` method of the `MarkdownCodec` `trait`.

- The `codec-markdown-trait` crate defines the `MarkdownCodec` `trait` which has the `to_markdown` method.

- The `codec-markdown-derive` crate provides a derive macro which is used to derive the `MarkdownCodec` trait for all types in the Stencila Schema.

It is necessary to have three separate crates because of the need to have a separate crate for derive macros and to avoid circular dependencies.

The `MarkdownCodec` derive macro has a `#[markdown(...)]` helper attribute which can be used to specify options for how the `to_markdown` method is derived for a type:

- `format`: A string, compatible with the [`format!` macro](https://doc.rust-lang.org/std/macro.format.html), which specifies how a node will be represented in Markdown

- `escape`: A character that should be escaped (with a single backslash) prior to formatting

- `special`: A boolean specifying whether a special, manually written function should be used for encoding the type to Markdown. If this is `true` then the type must implement a function named `to_markdown_special` with the same signature as the `to_markdown` method.

These options should be set in the `schema/*.yaml` files. These options will flow through to the `#[markdown(...)]` derive helper for the type when the files in `rust/schema/type` are regenerated. For example, the `schema/Strong.yaml` file contains the following:

```yaml
markdown:
  format: '**{content}**'
  escape: '*'
```

And the `schema/Heading.yaml` file contains the following:

```yaml
markdown:
  special: true
```

### Markdown to Stencila

The conversion from Markdown to a Stencila document is not yet re-implemented. The `v1` implementation, powered by `pulldown_cmark` and `nom` is [here](https://github.com/stencila/stencila/blob/v1/rust/codec-md/src/decode.rs).


## Encodings

### Inlines

#### Marks

`Emphasis` nodes are encoded using surrounding single underscores: `_content_`.

`Strong` nodes are encoded using surrounding double asterisks: `**content**`.

`Strikeout` nodes are encoded using surrounding double tildes: `~~content~~`.

`Subscript` nodes are encoded using surrounding single tildes: `~content~`.

`Superscript` nodes are encoded using surrounding single carets: `^content^`.

`Underline` nodes are encoded using bracketed spans with the `underline` keyword: `[content]{underline}`.

#### Quotes, links, and media objects

`Quote` nodes are encoded as HTML `<q>` elements.

`Link` nodes are encoded like so: `[content](target)` where `target` is the URL targeted by the link.

`ImageObject`, `AudioObject` and `VideoObject` nodes are all encoded like so: `![caption](contentUrl)`; during decoding the type is determined by the file extension of the `contentUrl`, falling back to `ImageObject`.

### Code and math fragments

`CodeFragment` nodes are surrounded by backticks: ``code``. If the `CodeFragment` has a programming language then it will be added within curly braces following the code: ``code`{programmingLanguage}`.

`MathFragment` nodes are encoded differently depending on the `mathLanguage`. If a node uses TeX it is encoded using surrounding dollar signs e.g. `$\pi$`. Otherwise, it will be surrounded by backticks with the language in curly braces (as for `CodeFragments`). e.g. AsciiMath `2 pi r^2`{asciimath}.


### Blocks

#### Math blocks

TeX `MathBlock` are encoded as Markdown paragraphs starting and ending with `$$` (no blank lines between them). e.g.

$$
2 \pi r^2
$$

Alternatively, code blocks with one of `asciimath`, `latex`, or `tex` as the language are interpreted as math blocks. e.g.

AsciiMath:

```asciimath
2 pi r^2
```

TeX:

```tex
2 \pi r^2
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

| Node type                                                                                                                 | Encoding         | Decoding      | Notes                             |
| ------------------------------------------------------------------------------------------------------------------------- | ---------------- | ------------- | --------------------------------- |
| **Works**                                                                                                                 |
| [Article](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/article.md)                          | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [AudioObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/audio_object.md)                 | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [Claim](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/claim.md)                              | ⚠️ High loss     |               | Encoded using special function    |
| [Collection](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/collection.md)                    | ⚠️ High loss     |               |                                   |
| [Comment](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/comment.md)                          | ⚠️ High loss     |               |                                   |
| [CreativeWork](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/creative_work.md)               | ⚠️ High loss     |               |                                   |
| [Directory](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/directory.md)                      | ⚠️ High loss     |               |                                   |
| [Figure](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/figure.md)                            | ⚠️ High loss     |               |                                   |
| [File](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/file.md)                                | ⚠️ High loss     |               |                                   |
| [ImageObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image_object.md)                 | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [MediaObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/media_object.md)                 | 🔷 Low loss       | 🔷 Low loss    |                                   |
| [Periodical](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/periodical.md)                    | ⚠️ High loss     |               |                                   |
| [PublicationIssue](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/publication_issue.md)       | ⚠️ High loss     |               |                                   |
| [PublicationVolume](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/publication_volume.md)     | ⚠️ High loss     |               |                                   |
| [Review](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/review.md)                            | ⚠️ High loss     |               |                                   |
| [SoftwareApplication](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software_application.md) | ⚠️ High loss     |               |                                   |
| [SoftwareSourceCode](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software_source_code.md)  | ⚠️ High loss     |               |                                   |
| [Table](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table.md)                              | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [TableCell](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table_cell.md)                     | 🔷 Low loss       | 🔷 Low loss    |                                   |
| [TableRow](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table_row.md)                       | 🔷 Low loss       | 🔷 Low loss    |                                   |
| [VideoObject](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/video_object.md)                 | ⚠️ High loss     |               | Encoded using special function    |
| **Prose**                                                                                                                 |
| [Admonition](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md)                    | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [Cite](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md)                                | ⚠️ High loss     |               |                                   |
| [CiteGroup](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite_group.md)                     | ⚠️ High loss     |               |                                   |
| [DefinedTerm](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/defined_term.md)                 | ⚠️ High loss     |               |                                   |
| [Delete](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/delete.md)                            | ⚠️ High loss     |               | Encoded as `<del>{content}</del>` |
| [Emphasis](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/emphasis.md)                        | 🟢 No loss        | 🟢 No loss     | Encoded as `_{content}_`          |
| [Heading](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md)                          | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [Insert](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/insert.md)                            | ⚠️ High loss     |               | Encoded as `<ins>{content}</ins>` |
| [Link](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md)                                | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [List](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md)                                | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [ListItem](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list_item.md)                       | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [Note](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/note.md)                                | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [Paragraph](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md)                      | 🟢 No loss        | 🟢 No loss     | Encoded as `{content}\n\n`        |
| [QuoteBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote_block.md)                   | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [QuoteInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote_inline.md)                 | ⚠️ High loss     |               | Encoded as `<q>{content}</q>`     |
| [Section](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md)                          | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [Strikeout](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strikeout.md)                      | ⚠️ High loss     |               | Encoded as `~~{content}~~`        |
| [Strong](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/strong.md)                            | 🟢 No loss        | 🟢 No loss     | Encoded as `**{content}**`        |
| [Subscript](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/subscript.md)                      | 🟢 No loss        | 🟢 No loss     | Encoded as `~{content}~`          |
| [Superscript](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/superscript.md)                  | 🟢 No loss        | 🟢 No loss     | Encoded as `^{content}^`          |
| [Text](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/text.md)                                | 🟢 No loss        | 🟢 No loss     | Encoded as `{value}`              |
| [ThematicBreak](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/thematic_break.md)             | 🟢 No loss        | 🟢 No loss     | Encoded as `***\n\n`              |
| [Underline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/underline.md)                      | 🟢 No loss        | 🟢 No loss     | Encoded as `<u>{content}</u>`     |
| **Math**                                                                                                                  |
| [MathBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math_block.md)                      | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [MathInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/math/math_inline.md)                    | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| **Code**                                                                                                                  |
| [CodeBlock](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_block.md)                      | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [CodeChunk](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_chunk.md)                      | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [CodeExpression](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_expression.md)            | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [CodeInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code_inline.md)                    | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [CompilationError](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/compilation_error.md)        | ⚠️ High loss     |               |                                   |
| [ExecutionError](https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution_error.md)            | ⚠️ High loss     |               |                                   |
| **Data**                                                                                                                  |
| [Array](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array.md)                               | ⚠️ High loss     |               |                                   |
| [ArrayValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array_validator.md)            | ⚠️ High loss     |               |                                   |
| [Boolean](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md)                           | 🔷 Low loss       | 🔷 Low loss    |                                   |
| [BooleanValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean_validator.md)        | ⚠️ High loss     |               |                                   |
| [ConstantValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/constant_validator.md)      | ⚠️ High loss     |               |                                   |
| [Cord](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/cord.md)                                 | 🟢 No loss        | 🟢 No loss     |                                   |
| [Datatable](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable.md)                       | ⚠️ High loss     |               |                                   |
| [DatatableColumn](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable_column.md)          | ⚠️ High loss     |               |                                   |
| [Date](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date.md)                                 | ⚠️ High loss     |               |                                   |
| [DateTime](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_time.md)                        | ⚠️ High loss     |               |                                   |
| [DateTimeValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_time_validator.md)     | ⚠️ High loss     |               |                                   |
| [DateValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/date_validator.md)              | ⚠️ High loss     |               |                                   |
| [Duration](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration.md)                         | ⚠️ High loss     |               |                                   |
| [DurationValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/duration_validator.md)      | ⚠️ High loss     |               |                                   |
| [EnumValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/enum_validator.md)              | ⚠️ High loss     |               |                                   |
| [Integer](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer.md)                           | 🔷 Low loss       | 🔷 Low loss    |                                   |
| [IntegerValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/integer_validator.md)        | ⚠️ High loss     |               |                                   |
| [Null](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/null.md)                                 | 🔷 Low loss       | 🔷 Low loss    |                                   |
| [Number](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md)                             | 🔷 Low loss       | 🔷 Low loss    |                                   |
| [NumberValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number_validator.md)          | ⚠️ High loss     |               |                                   |
| [Object](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/object.md)                             | ⚠️ High loss     |               |                                   |
| [String](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md)                             | 🟢 No loss        | 🟢 No loss     |                                   |
| [StringValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string_validator.md)          | ⚠️ High loss     |               |                                   |
| [Time](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time.md)                                 | ⚠️ High loss     |               |                                   |
| [TimeValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/time_validator.md)              | ⚠️ High loss     |               |                                   |
| [Timestamp](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp.md)                       | ⚠️ High loss     |               |                                   |
| [TimestampValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/timestamp_validator.md)    | ⚠️ High loss     |               |                                   |
| [TupleValidator](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/tuple_validator.md)            | ⚠️ High loss     |               |                                   |
| [UnsignedInteger](https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned_integer.md)          | 🔷 Low loss       | 🔷 Low loss    |                                   |
| **Flow**                                                                                                                  |
| [Button](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/button.md)                             | ⚠️ High loss     |               |                                   |
| [Call](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call.md)                                 | ⚠️ High loss     |               | Encoded using special function    |
| [CallArgument](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call_argument.md)                | ⚠️ High loss     |               |                                   |
| [CodeLocation](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/code_location.md)                | ⚠️ High loss     |               |                                   |
| [CompilationDigest](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/compilation_digest.md)      | ⚠️ High loss     |               |                                   |
| [ExecutionDependant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_dependant.md)    | ⚠️ High loss     |               |                                   |
| [ExecutionDependency](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_dependency.md)  | ⚠️ High loss     |               |                                   |
| [ExecutionTag](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution_tag.md)                | ⚠️ High loss     |               |                                   |
| [For](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for.md)                                   | ⚠️ High loss     |               | Encoded using special function    |
| [Form](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/form.md)                                 | ⚠️ High loss     |               |                                   |
| [Function](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/function.md)                         | ⚠️ High loss     |               |                                   |
| [If](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if.md)                                     | ⚠️ High loss     |               | Encoded using special function    |
| [IfClause](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if_clause.md)                        | ⚠️ High loss     |               |                                   |
| [Include](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include.md)                           | ⚠️ High loss     |               | Encoded as `/{source}\n\n`        |
| [Parameter](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/parameter.md)                       | 🔷 Low loss       | 🔷 Low loss    | Encoded using special function    |
| [Variable](https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/variable.md)                         | ⚠️ High loss     |               |                                   |
| **Style**                                                                                                                 |
| [Division](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/division.md)                        | 🟢 No loss        | 🟢 No loss     | Encoded using special function    |
| [StyledInline](https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/styled_inline.md)               | ⚠️ High loss     |               | Encoded using special function    |
| **Other**                                                                                                                 |
| [Brand](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/brand.md)                              | ⚠️ High loss     |               |                                   |
| [ContactPoint](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/contact_point.md)               | ⚠️ High loss     |               |                                   |
| [Enumeration](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/enumeration.md)                  | ⚠️ High loss     |               |                                   |
| [Grant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/grant.md)                              | ⚠️ High loss     |               |                                   |
| [MonetaryGrant](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/monetary_grant.md)             | ⚠️ High loss     |               |                                   |
| [Organization](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md)                | ⚠️ High loss     |               |                                   |
| [Person](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md)                            | ⚠️ High loss     |               |                                   |
| [PostalAddress](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/postal_address.md)             | ⚠️ High loss     |               |                                   |
| [Product](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/product.md)                          | ⚠️ High loss     |               |                                   |
| [PropertyValue](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/property_value.md)             | ⚠️ High loss     |               |                                   |
| [Thing](https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/thing.md)                              | ⚠️ High loss     |               |                                   |

<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
