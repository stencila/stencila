<!-- Generated from site/docs/documents by claude/scripts/gen-references.sh. Do not edit. -->


# Basics

This page covers the everyday document features that most Stencila documents start with: paragraphs, headings, inline formatting, links, inline code, quotes, and thematic breaks.

Although these features will be familiar from Markdown, Stencila parses them into structured document nodes and preserves that structure across editing, execution, conversion, and publishing workflows.

# Paragraphs

Paragraphs are the basic unit of prose. Separate them with a blank line:

```smd
This is paragraph one. It has two sentences.

Paragraph two.
```

Most of the inline syntax on this page is written inside paragraphs.

# Headings

Use hash prefixes to create headings from levels 1 to 6:

```smd
# Level 1

## Level 2

### Level 3

#### Level 4

##### Level 5

###### Level 6
```

Headings provide the main outline for your document and are commonly used to structure sections.

# Inline formatting

Stencila Markdown supports several common inline formatting forms.

## Emphasis, strong, and underline

Use `_..._` for emphasis, `**...**` for strong emphasis, and HTML `<u>...</u>` for underline:

```smd
This is _emphasized_, this is **strongly emphasized**, and this is <u>underlined</u>.
```

## Strikeout, subscript, and superscript

Use `~~...~~` for strikeout, `~...~` for subscript, and `^...^` for superscript:

```smd
This is ~subscripted~, this is ^superscripted^, and this is ~~struck out~~.
```

## Escaping formatting markers

When you want literal formatting characters instead of formatting, escape them with backslashes:

```smd
_\_ under\_scores \__, and **\* aster\*isks \***.
```

# Links

Use standard Markdown link syntax:

```smd
A link [example.org](https://example.org).
```

Links can also include titles, and the link text can contain inline formatting:

```smd
With a title and styled content [**example.org**](https://example.org "a title").
```

# Inline code

Use backticks for short code fragments:

```smd
A code fragment: `2 * pi * r^2`.
```

You can also add a language tag to an inline code fragment when you want to make the language explicit:

```smd
With language: `6 * 7`{python}.
```

> [!tip]
> If you want executable inline code, see [Execution](execution).

# Inline quotes

For short inline quotations, you can use HTML `<q>` when you want quotation semantics inside a paragraph:

```smd
Stencila follows the Robustness Principle: <q>be conservative in what you do, be liberal in what you accept from others</q>.
```

# Block quotes

Use `>` to create quoted block content:

```smd
> This is a quote block.
```

Quote blocks can contain multiple paragraphs:

```smd
> This quote block...
>
> Has two paragraphs.
```

They can also be nested:

```smd
> This quote block...
>
> > Has another inside it.
> >
> > > And another inside that!
```

# Thematic breaks

Use three or more asterisks on their own line to create a thematic break:

```smd
A paragraph before a thematic break.

***

A paragraph after a thematic break.
```

Thematic breaks are useful for marking a visible shift in topic or separating major parts of a document.

