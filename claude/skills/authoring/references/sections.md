<!-- Generated from site/docs/documents by claude/scripts/gen-references.sh. Do not edit. -->


# Sections

Sections organize a document into structural parts such as introductions, methods, results, discussions, appendices, headers, and footers.

Stencila supports both generic sections and recognized named section types.

# Generic sections

Use `::: section` to create an explicit section container:

```smd
::: section

# A section heading

Section content.

:::
```

Explicit sections are useful when you want a structural container in the document model, rather than relying only on heading levels.

# Nested sections

Sections can be nested inside one another:

```smd
::: section

# Nested sections

::: section

## Inner section

::: section

### First inner-inner section

This is the first inner-inner section

:::

::: section

### Second inner-inner section

This is the second inner-inner section

:::

:::

:::
```

# Recognized section types

Stencila recognizes many common document section types and maps them to a structured `sectionType` value. These types are based largely on section categories used in scholarly and technical documents.

For example:

```smd
::: header

This is the header

:::

::: introduction

This is the introduction

:::

::: discussion

This is the discussion

:::

::: footer

This is the footer

:::
```

The examples above are recognized as typed sections rather than just generic containers.

# Common section types

Some of the section types Stencila recognizes include:

- `abstract`
- `summary`
- `highlights`
- `introduction`
- `background`
- `related-work`
- `materials`
- `methods`
- `experimental-design`
- `statistical-analysis`
- `results`
- `discussion`
- `limitations`
- `conclusions`
- `future-work`
- `references`
- `acknowledgements`
- `funding`
- `competing-interests`
- `ethics`
- `author-contributions`
- `data-availability`
- `code-availability`
- `reproducibility`
- `abbreviations`
- `nomenclature`
- `supplementary-materials`
- `appendix`
- `main`
- `header`
- `footer`

These recognized section types help preserve document structure across formats and make documents more semantically explicit.

# Headings and sections

Headings and sections are related but not identical:

- a **heading** is a title line such as `# Introduction`
- a **section** is a structural container that can contain headings and other content

Many documents use both together, for example an `::: introduction` section containing an `# Introduction` heading.

# Appendix breaks

Use `::: appendix` to mark the transition into appendices:

Unlike `::: section`, `::: appendix` is a break marker rather than a container, so it does not need a closing `:::`.

```smd
A paragraph before first appendix break.

::: appendix

A paragraph after first appendix break.
```

Appendix breaks are useful when a document transitions from the main text into appendix material.

> [!tip]
> If you are trying to add titles, authors, dates, keywords, or other article-level properties, see [Metadata](metadata).

