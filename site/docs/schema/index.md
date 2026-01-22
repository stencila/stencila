---
title: Stencila Schema
---

Stencila Schema is the canonical model for representing documents, data, code, and
execution in Stencila. It defines the node types that flow through parsers,
codecs, editors, kernels, and publishing pipelines so that content can be
validated, transformed, and exchanged consistently across formats and languages.

# Overview

The schema is authored as YAML files in `schema/`, with each file describing a
type, its properties, inheritance, and metadata such as JSON-LD identifiers. The
result is an interconnected graph of node types covering creative works, prose,
math, code, data structures, execution flow, styling, edits, and configuration.

## Why it exists

Stencila supports many input and output formats. A shared schema provides a
single source of truth so that:

- documents can round-trip between formats with fewer surprises
- validation rules and constraints are centralized
- type-safe APIs can be generated for multiple languages
- semantic identifiers align with JSON-LD and https://schema.org conventions

## Relationship to schema.org

Where possible, Stencila adopts schema.org types and properties so that metadata
and content are immediately interoperable with existing tools, crawlers, and
linked data ecosystems. When schema.org lacks the concepts needed for
executable, interactive, or richly structured documents, Stencila extends it
with additional types and properties rather than inventing a parallel
vocabulary. This keeps common concepts aligned with the wider web while
allowing Stencila to model features such as execution, provenance, prompts, and
structured data.

## Generated artifacts

The Rust crate in `rust/schema-gen/` is responsible for turning the YAML source
files into concrete outputs used throughout the project. It reads the schema,
checks and extends inheritance, expands union types, and then generates:

- reference documentation in `site/docs/schema/` (this section)
- codec capability documentation in `docs/formats/`
- Rust types in `rust/schema/src/types.rs`
- TypeScript types in `ts/src/types/` and supporting enums in `ts/src/`
- Python types in `python/stencila_types/src/stencila_types/types.py`
- JSON Schema files in `json/*.schema.json`
- JSON-LD contexts in `json/*.jsonld`
- Kuzu graph schema, migrations, and Rust ORM code in `rust/node-db/`

## Using this reference

Reference documentation for Stencila Schema types is organized by category
below. Each type page is generated from its corresponding YAML schema and
includes the full set of properties and constraints.

***

# Works

- [`Article`](./article.md) - An article, including news and scholarly articles.
- [`AudioObject`](./audio-object.md) - An audio file.
- [`Author`](./author.md) - Union type for things that can be an author of a `CreativeWork` or other type.
- [`AuthorRole`](./author-role.md) - An author and their role.
- [`AuthorRoleAuthor`](./author-role-author.md) - Union type for things that can be an author in `AuthorRole`.
- [`AuthorRoleName`](./author-role-name.md) - A `roleName` for an `AuthorRole`.
- [`Bibliography`](./bibliography.md) - A bibliography loaded from an external source file.
- [`Chat`](./chat.md) - A chat conversation, usually with a generative AI model.
- [`ChatMessage`](./chat-message.md) - A message within a `Chat`.
- [`ChatMessageGroup`](./chat-message-group.md) - A group of messages, usually alternative model messages, within a `Chat`.
- [`Claim`](./claim.md) - A claim represents specific reviewable facts or statements.
- [`ClaimType`](./claim-type.md) - The type of a `Claim`.
- [`Collection`](./collection.md) - A collection of CreativeWorks or other artifacts.
- [`Comment`](./comment.md) - A comment on an item, e.g on a `Article` or `SoftwareSourceCode`.
- [`CreativeWork`](./creative-work.md) - A creative work, including books, movies, photographs, software programs, etc.
- [`CreativeWorkType`](./creative-work-type.md) - The kind of a creative work.
- [`CreativeWorkVariant`](./creative-work-variant.md) - Union type for all types that are descended from `CreativeWork`
- [`Directory`](./directory.md) - A directory on the file system.
- [`Figure`](./figure.md) - Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them.
- [`File`](./file.md) - A file on the file system.
- [`HorizontalAlignment`](./horizontal-alignment.md) - The horizontal alignment of content.
- [`ImageObject`](./image-object.md) - An image file.
- [`InstructionType`](./instruction-type.md) - The type of an instruction describing the operation to be performed.
- [`LabelType`](./label-type.md) - Indicates how a block (usually a `CodeChunk`) should be automatically labelled.
- [`MediaObject`](./media-object.md) - A media object, such as an image, video, or audio object embedded in a web page or a downloadable dataset.
- [`Periodical`](./periodical.md) - A periodical publication.
- [`Prompt`](./prompt.md) - A prompt for creating or editing document content.
- [`PublicationIssue`](./publication-issue.md) - A part of a successively published publication such as a periodical or publication volume, often numbered.
- [`PublicationVolume`](./publication-volume.md) - A part of a successively published publication such as a periodical or multi-volume work.
- [`Reference`](./reference.md) - A reference to a creative work, including books, movies, photographs, software programs, etc.
- [`Review`](./review.md) - A review of an item, e.g of an `Article` or `SoftwareApplication`.
- [`SoftwareApplication`](./software-application.md) - A software application.
- [`SoftwareSourceCode`](./software-source-code.md) - Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
- [`Supplement`](./supplement.md) - A supplementary `CreativeWork` that supports this work but is not considered part of its main content.
- [`Table`](./table.md) - A table.
- [`TableCell`](./table-cell.md) - A cell within a `Table`.
- [`TableCellType`](./table-cell-type.md) - Indicates whether the cell is a header or data.
- [`TableRow`](./table-row.md) - A row within a Table.
- [`TableRowType`](./table-row-type.md) - Indicates whether the row is in the header, body or footer of the table.
- [`VerticalAlignment`](./vertical-alignment.md) - The vertical alignment of content.
- [`VideoObject`](./video-object.md) - A video file.

# Prose

- [`Admonition`](./admonition.md) - An admonition within a document.
- [`AdmonitionType`](./admonition-type.md) - The type of an `Admonition`.
- [`Annotation`](./annotation.md) - Annotated content.
- [`AppendixBreak`](./appendix-break.md) - A break in a document indicating the start one or more appendices.
- [`Block`](./block.md) - Union type in block content node types.
- [`Citation`](./citation.md) - A reference to a `CreativeWork` that is cited in another `CreativeWork`.
- [`CitationGroup`](./citation-group.md) - A group of `Citation` nodes.
- [`CitationIntent`](./citation-intent.md) - The type or nature of a citation, both factually and rhetorically.
- [`CitationMode`](./citation-mode.md) - The mode of a `Citation`.
- [`DefinedTerm`](./defined-term.md) - A word, name, acronym, phrase, etc. with a formal definition.
- [`Emphasis`](./emphasis.md) - Emphasized content.
- [`Heading`](./heading.md) - A heading.
- [`Inline`](./inline.md) - Union type for valid inline content.
- [`InlinesBlock`](./inlines-block.md) - A block containing inlines with no other semantics.
- [`Link`](./link.md) - A hyperlink to other pages, sections within the same document, resources, or any URL.
- [`List`](./list.md) - A list of items.
- [`ListItem`](./list-item.md) - A single item in a list.
- [`ListOrder`](./list-order.md) - Indicates how a `List` is ordered.
- [`Mark`](./mark.md) - Abstract base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).
- [`Note`](./note.md) - Additional content which is not part of the main content of a document.
- [`NoteType`](./note-type.md) - The type of a `Note` which determines where the note content is displayed within the document.
- [`Paragraph`](./paragraph.md) - A paragraph.
- [`QuoteBlock`](./quote-block.md) - A section quoted from somewhere else.
- [`QuoteInline`](./quote-inline.md) - Inline, quoted content.
- [`Section`](./section.md) - A section of a document.
- [`SectionType`](./section-type.md) - The type of a `Section`.
- [`Sentence`](./sentence.md) - A sentence, usually within a `Paragraph`.
- [`Strikeout`](./strikeout.md) - Content that is marked as struck out.
- [`Strong`](./strong.md) - Strongly emphasized content.
- [`Subscript`](./subscript.md) - Subscripted content.
- [`Superscript`](./superscript.md) - Superscripted content.
- [`Text`](./text.md) - Textual content.
- [`ThematicBreak`](./thematic-break.md) - A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
- [`Underline`](./underline.md) - Inline text that is underlined.

# Math

- [`Math`](./math.md) - Abstract base type for a mathematical variable or equation.
- [`MathBlock`](./math-block.md) - A block of math, e.g an equation, to be treated as block content.
- [`MathInline`](./math-inline.md) - A fragment of math, e.g a variable name, to be treated as inline content.

# Code

- [`CodeBlock`](./code-block.md) - A code block.
- [`CodeChunk`](./code-chunk.md) - A executable chunk of code.
- [`CodeExecutable`](./code-executable.md) - Abstract base type for executable code nodes (e.g. `CodeChunk`).
- [`CodeExpression`](./code-expression.md) - An executable code expression.
- [`CodeInline`](./code-inline.md) - Inline code.
- [`CodeStatic`](./code-static.md) - Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
- [`CompilationMessage`](./compilation-message.md) - An error, warning or log message generated during compilation.
- [`ExecutionMessage`](./execution-message.md) - An error, warning or log message generated during execution.

# Data

- [`Array`](./array.md) - A value comprised of other primitive nodes.
- [`ArrayHint`](./array-hint.md) - A hint to the content of an `Array`.
- [`ArrayValidator`](./array-validator.md) - A validator specifying constraints on an array node.
- [`Boolean`](./boolean.md) - A value that is either true or false.
- [`BooleanValidator`](./boolean-validator.md) - A schema specifying that a node must be a boolean value.
- [`ConstantValidator`](./constant-validator.md) - A validator specifying a constant value that a node must have.
- [`Cord`](./cord.md) - A value comprised of a sequence of characters.
- [`Datatable`](./datatable.md) - A table of data.
- [`DatatableColumn`](./datatable-column.md) - A column of data within a `Datatable`.
- [`DatatableColumnHint`](./datatable-column-hint.md) - A hint to the type and values in a `DatatableColumn`.
- [`DatatableHint`](./datatable-hint.md) - A hint to the structure of a table of data.
- [`Date`](./date.md) - A calendar date encoded as a ISO 8601 string.
- [`DateTime`](./date-time.md) - A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
- [`DateTimeValidator`](./date-time-validator.md) - A validator specifying the constraints on a date-time.
- [`DateValidator`](./date-validator.md) - A validator specifying the constraints on a date.
- [`Duration`](./duration.md) - A value that represents the difference between two timestamps.
- [`DurationValidator`](./duration-validator.md) - A validator specifying the constraints on a duration.
- [`EnumValidator`](./enum-validator.md) - A schema specifying that a node must be one of several values.
- [`Hint`](./hint.md) - Union type for hints of the value and/or structure of data.
- [`Integer`](./integer.md) - A value that is a integer.
- [`IntegerValidator`](./integer-validator.md) - A validator specifying the constraints on an integer node.
- [`Null`](./null.md) - The null value.
- [`Number`](./number.md) - A value that is a number.
- [`NumberValidator`](./number-validator.md) - A validator specifying the constraints on a numeric node.
- [`Object`](./object.md) - A value comprised of keyed primitive nodes.
- [`ObjectHint`](./object-hint.md) - A hint to the structure of an `Object`.
- [`Primitive`](./primitive.md) - Union type for all primitives values.
- [`String`](./string.md) - A value comprised of a string of characters.
- [`StringHint`](./string-hint.md) - A hint to the structure of an `String`.
- [`StringValidator`](./string-validator.md) - A schema specifying constraints on a string node.
- [`Time`](./time.md) - A point in time recurring on multiple days.
- [`TimeUnit`](./time-unit.md) - A unit in which time can be measured.
- [`TimeValidator`](./time-validator.md) - A validator specifying the constraints on a time.
- [`Timestamp`](./timestamp.md) - A value that represents a point in time.
- [`TimestampValidator`](./timestamp-validator.md) - A validator specifying the constraints on a timestamp.
- [`TupleValidator`](./tuple-validator.md) - A validator specifying constraints on an array of heterogeneous items.
- [`Unknown`](./unknown.md) - A type to indicate a value or or other type in unknown.
- [`UnsignedInteger`](./unsigned-integer.md) - An integer value that is greater or equal to zero.
- [`Validator`](./validator.md) - Union type for validators.

# Flow

- [`Button`](./button.md) - A button.
- [`CallArgument`](./call-argument.md) - The value of a `Parameter` to call a document with.
- [`CallBlock`](./call-block.md) - Call another document, optionally with arguments, and include its executed content.
- [`CodeLocation`](./code-location.md) - The location within some source code.
- [`CompilationDigest`](./compilation-digest.md) - A digest of the content, semantics and dependencies of an executable node.
- [`Executable`](./executable.md) - Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).
- [`ExecutionBounds`](./execution-bounds.md) - The bounds placed on the execution of a document node.
- [`ExecutionDependant`](./execution-dependant.md) - A downstream execution dependant of a node.
- [`ExecutionDependantRelation`](./execution-dependant-relation.md) - The relation between a node and its execution dependant.
- [`ExecutionDependency`](./execution-dependency.md) - An upstream execution dependency of a node.
- [`ExecutionDependencyRelation`](./execution-dependency-relation.md) - The relation between a node and its execution dependency.
- [`ExecutionMode`](./execution-mode.md) - Under which circumstances a node should be executed.
- [`ExecutionRequired`](./execution-required.md) - Whether, and why, the execution of a node is required or not.
- [`ExecutionStatus`](./execution-status.md) - Status of the most recent, including any current, execution of a document node.
- [`ExecutionTag`](./execution-tag.md) - A tag on code that affects its execution.
- [`ForBlock`](./for-block.md) - Repeat a block content for each item in an array.
- [`Form`](./form.md) - A form to batch updates in document parameters.
- [`FormDeriveAction`](./form-derive-action.md) - Indicates the action (create, update or delete) to derive for a `Form`.
- [`Function`](./function.md) - A function with a name, which might take Parameters and return a value of a certain type.
- [`IfBlock`](./if-block.md) - Show and execute alternative content conditional upon an executed expression.
- [`IfBlockClause`](./if-block-clause.md) - A clause within an `IfBlock` node.
- [`IncludeBlock`](./include-block.md) - Include block content from an external source (e.g. file, URL).
- [`Parameter`](./parameter.md) - A parameter of a document.
- [`Variable`](./variable.md) - A variable representing a name / value pair.
- [`Walkthrough`](./walkthrough.md) - An interactive walkthrough made up of several, successively revealed steps.
- [`WalkthroughStep`](./walkthrough-step.md) - A step in a walkthrough.

# Style

- [`Page`](./page.md) - A separate page in a document
- [`Styled`](./styled.md) - An abstract base class for a document node that has styling applied to it and/or its content.
- [`StyledBlock`](./styled-block.md) - Styled block content.
- [`StyledInline`](./styled-inline.md) - Styled inline content.

# Edits

- [`Instruction`](./instruction.md) - Abstract base type for a document editing instruction.
- [`InstructionBlock`](./instruction-block.md) - An instruction to edit some block content.
- [`InstructionInline`](./instruction-inline.md) - An instruction to edit some inline content.
- [`InstructionMessage`](./instruction-message.md) - A message within an `Instruction`.
- [`MessagePart`](./message-part.md) - A union type for a part of a message.
- [`PromptBlock`](./prompt-block.md) - A preview of how a prompt will be rendered at a location in the document
- [`Suggestion`](./suggestion.md) - Abstract base type for nodes that indicate a suggested change to content.
- [`SuggestionBlock`](./suggestion-block.md) - Abstract base type for nodes that indicate a suggested change to block content.
- [`SuggestionInline`](./suggestion-inline.md) - Abstract base type for nodes that indicate a suggested change to inline content.
- [`SuggestionStatus`](./suggestion-status.md) - The status of an instruction.

# Config

- [`Config`](./config.md) - Stencila document configuration options.
- [`ConfigModels`](./config-models.md) - Model selection and execution options.
- [`ConfigPublish`](./config-publish.md) - Publishing options.
- [`ConfigPublishGhost`](./config-publish-ghost.md) - Ghost publishing options.
- [`ConfigPublishGhostState`](./config-publish-ghost-state.md) - The state of Ghost resource
- [`ConfigPublishGhostType`](./config-publish-ghost-type.md) - The type of Ghost resource
- [`ConfigPublishZenodo`](./config-publish-zenodo.md) - Zenodo publishing options.
- [`ConfigPublishZenodoAccessRight`](./config-publish-zenodo-access-right.md) - The access right type

# Other

- [`Brand`](./brand.md) - A brand used by an organization or person for labeling a product, product group, or similar.
- [`ContactPoint`](./contact-point.md) - A contact point, usually within an organization.
- [`Entity`](./entity.md) - Abstract base type for compound (ie. non-atomic) nodes.
- [`Enumeration`](./enumeration.md) - Lists or enumerations, for example, a list of cuisines or music genres, etc.
- [`Excerpt`](./excerpt.md) - An excerpt from a `CreativeWork`.
- [`Grant`](./grant.md) - A grant, typically financial or otherwise quantifiable, of resources.
- [`Island`](./island.md) - An island of content in a document.
- [`MessageLevel`](./message-level.md) - The severity level of a message.
- [`MessageRole`](./message-role.md) - The role of a message.
- [`ModelParameters`](./model-parameters.md) - Model selection and inference parameters for generative AI models.
- [`MonetaryGrant`](./monetary-grant.md) - A monetary grant.
- [`Node`](./node.md) - Union type for all types in this schema, including primitives and entities
- [`Organization`](./organization.md) - An organization such as a school, NGO, corporation, club, etc.
- [`Person`](./person.md) - A person (alive, dead, undead, or fictional).
- [`PostalAddress`](./postal-address.md) - A physical mailing address.
- [`Product`](./product.md) - Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
- [`PropertyValue`](./property-value.md) - A property-value pair.
- [`ProvenanceCategory`](./provenance-category.md) - A description of the provenance of content in terms of human/machine involvement.
- [`ProvenanceCount`](./provenance-count.md) - The count of the number of characters in a `ProvenanceCategory` within an entity.
- [`RawBlock`](./raw-block.md) - Document content in a specific format
- [`RelativePosition`](./relative-position.md) - The relative position of a node to another.
- [`Role`](./role.md) - Represents additional information about a relationship or property.
- [`Thing`](./thing.md) - The most generic type of item.
- [`ThingVariant`](./thing-variant.md) - Union type for all types that are descended from `Thing`
