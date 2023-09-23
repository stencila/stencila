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
<!-- CODEC-DOCS:STOP -->
<!-- prettier-ignore-end -->
