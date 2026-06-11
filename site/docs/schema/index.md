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

## Using this reference

Reference documentation for Stencila Schema types is organized by category
below. Each type page is generated from its corresponding YAML schema and
includes the full set of properties and constraints.

***


# Works

- [`Agent`](./agent.md) - An AI agent definition.
- [`Article`](./article.md) - An article, including news and scholarly articles.
- [`AudioObject`](./audio-object.md) - An audio file.
- [`Author`](./author.md) - A union type for authors of a `CreativeWork` or other type.
- [`AuthorRole`](./author-role.md) - An author and their role.
- [`AuthorRoleAuthor`](./author-role-author.md) - A union type for authors in an `AuthorRole`.
- [`AuthorRoleName`](./author-role-name.md) - A controlled author contribution role.
- [`Bibliography`](./bibliography.md) - A bibliography loaded from an external source file.
- [`Chat`](./chat.md) - A chat conversation, usually with a generative AI model.
- [`ChatMessage`](./chat-message.md) - A message within a `Chat`.
- [`ChatMessageGroup`](./chat-message-group.md) - A group of messages, usually alternative model messages, within a `Chat`.
- [`Claim`](./claim.md) - A reviewable claim or statement.
- [`ClaimType`](./claim-type.md) - A category of claim.
- [`Collection`](./collection.md) - A collection of creative works or other artifacts.
- [`Comment`](./comment.md) - A comment on an item.
- [`ContainerImage`](./container-image.md) - A container image used by a computational action.
- [`CreativeWork`](./creative-work.md) - A creative work.
- [`CreativeWorkType`](./creative-work-type.md) - A category of creative work.
- [`CreativeWorkVariant`](./creative-work-variant.md) - Union type for all types that are descended from `CreativeWork`
- [`Directory`](./directory.md) - A directory on a file system.
- [`Evidence`](./evidence.md) - Evidence supporting, opposing, or otherwise informing a research claim.
- [`Figure`](./figure.md) - A figure.
- [`File`](./file.md) - A file on the file system.
- [`HorizontalAlignment`](./horizontal-alignment.md) - The horizontal alignment of content.
- [`ImageObject`](./image-object.md) - An image file.
- [`InstructionType`](./instruction-type.md) - An operation requested by an instruction.
- [`LabelType`](./label-type.md) - An automatic labeling category.
- [`MediaObject`](./media-object.md) - A media object.
- [`Periodical`](./periodical.md) - A periodical publication.
- [`Prompt`](./prompt.md) - A prompt for creating or editing document content.
- [`Protocol`](./protocol.md) - A research protocol or method description.
- [`PublicationIssue`](./publication-issue.md) - A publication issue.
- [`PublicationVolume`](./publication-volume.md) - A publication volume.
- [`Question`](./question.md) - A question or research prompt.
- [`Reference`](./reference.md) - A reference to a creative work, including books, movies, photographs, software programs, etc.
- [`Request`](./request.md) - A request for research work, evidence, protocol execution, or another contribution.
- [`ResearchObject`](./research-object.md) - An abstract base type for research objects represented as block content.
- [`ResearchObjectRelation`](./research-object-relation.md) - A relation from one research object to another.
- [`ResearchObjectRelationKind`](./research-object-relation-kind.md) - The kind of relation from one research object to another.
- [`Review`](./review.md) - A review of an item, e.g of an `Article` or `SoftwareApplication`.
- [`Skill`](./skill.md) - An agent skill providing instructions for AI agents.
- [`SoftwareApplication`](./software-application.md) - A software application.
- [`SoftwareSourceCode`](./software-source-code.md) - Source code for software.
- [`Supplement`](./supplement.md) - A supplementary creative work associated with a document.
- [`SymbolicLink`](./symbolic-link.md) - A symbolic link on a file system.
- [`Table`](./table.md) - A table.
- [`TableCell`](./table-cell.md) - A cell within a `Table`.
- [`TableCellType`](./table-cell-type.md) - The structural role of a table cell.
- [`TableRow`](./table-row.md) - A row within a Table.
- [`TableRowType`](./table-row-type.md) - The structural role of a table row.
- [`VerticalAlignment`](./vertical-alignment.md) - The vertical alignment of content.
- [`VideoObject`](./video-object.md) - A video file.
- [`Workflow`](./workflow.md) - An AI workflow definition.
- [`WorktreeStatus`](./worktree-status.md) - The status of a source worktree relative to a commit.

# Prose

- [`Admonition`](./admonition.md) - An admonition within a document.
- [`AdmonitionType`](./admonition-type.md) - A category of admonition.
- [`Annotation`](./annotation.md) - Annotated content.
- [`AppendixBreak`](./appendix-break.md) - A break marking the start of appendices.
- [`Block`](./block.md) - A union type for block content.
- [`Boundary`](./boundary.md) - A positional boundary marker within inline content.
- [`Citation`](./citation.md) - A reference to a `CreativeWork` that is cited in another `CreativeWork`.
- [`CitationGroup`](./citation-group.md) - A group of `Citation` nodes.
- [`CitationIntent`](./citation-intent.md) - The rhetorical or factual intent of a citation.
- [`CitationMode`](./citation-mode.md) - The presentation mode of a citation.
- [`DefinedTerm`](./defined-term.md) - A word, name, acronym, phrase, etc. with a formal definition.
- [`Emphasis`](./emphasis.md) - Emphasized content.
- [`Heading`](./heading.md) - A heading.
- [`Icon`](./icon.md) - An icon, typically rendered using an icon font.
- [`Inline`](./inline.md) - Union type for valid inline content.
- [`InlinesBlock`](./inlines-block.md) - A block containing inlines with no other semantics.
- [`Link`](./link.md) - A hyperlink to other pages, sections within the same document, resources, or any URL.
- [`List`](./list.md) - A list of items.
- [`ListItem`](./list-item.md) - A single item in a list.
- [`ListOrder`](./list-order.md) - The ordering of a list.
- [`Mark`](./mark.md) - An abstract base type for marked inline content.
- [`Note`](./note.md) - A note associated with document content.
- [`NoteType`](./note-type.md) - A category of note placement.
- [`Paragraph`](./paragraph.md) - A paragraph.
- [`QuoteBlock`](./quote-block.md) - A section quoted from somewhere else.
- [`QuoteInline`](./quote-inline.md) - Inline, quoted content.
- [`Section`](./section.md) - A section of a document.
- [`SectionType`](./section-type.md) - A category of section.
- [`Sentence`](./sentence.md) - A sentence, usually within a `Paragraph`.
- [`Strikeout`](./strikeout.md) - Content that is marked as struck out.
- [`Strong`](./strong.md) - Strongly emphasized content.
- [`Subscript`](./subscript.md) - Subscripted content.
- [`Superscript`](./superscript.md) - Superscripted content.
- [`Text`](./text.md) - A text node.
- [`ThematicBreak`](./thematic-break.md) - A thematic break.
- [`Underline`](./underline.md) - Inline text that is underlined.

# Math

- [`Math`](./math.md) - An abstract base type for mathematical content.
- [`MathBlock`](./math-block.md) - A block of math, e.g an equation, to be treated as block content.
- [`MathInline`](./math-inline.md) - A fragment of math, e.g a variable name, to be treated as inline content.

# Code

- [`CodeBlock`](./code-block.md) - A code block.
- [`CodeChunk`](./code-chunk.md) - An executable code chunk.
- [`CodeExecutable`](./code-executable.md) - An abstract base type for executable code nodes.
- [`CodeExpression`](./code-expression.md) - An executable code expression.
- [`CodeInline`](./code-inline.md) - Inline code.
- [`CodeStatic`](./code-static.md) - An abstract base type for non-executable code nodes.
- [`CompilationMessage`](./compilation-message.md) - An error, warning or log message generated during compilation.
- [`ExecutionMessage`](./execution-message.md) - An error, warning or log message generated during execution.

# Data

- [`Array`](./array.md) - An array value.
- [`ArrayHint`](./array-hint.md) - A concise summary of the values and structure of an `Array`.
- [`ArrayValidator`](./array-validator.md) - A validator specifying constraints on an array node.
- [`Boolean`](./boolean.md) - A value that is either true or false.
- [`BooleanValidator`](./boolean-validator.md) - A validator for boolean values.
- [`ConstantValidator`](./constant-validator.md) - A validator specifying a constant value that a node must have.
- [`Cord`](./cord.md) - A CRDT-backed sequence of characters.
- [`Datatable`](./datatable.md) - A table of data.
- [`DatatableColumn`](./datatable-column.md) - A column of data within a `Datatable`.
- [`DatatableColumnHint`](./datatable-column-hint.md) - A concise summary of the properties of a `DatatableColumn`.
- [`DatatableHint`](./datatable-hint.md) - A concise summary of the structure of a table of data.
- [`Date`](./date.md) - A calendar date encoded as a ISO 8601 string.
- [`DateTime`](./date-time.md) - A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
- [`DateTimeValidator`](./date-time-validator.md) - A validator specifying the constraints on a date-time.
- [`DateValidator`](./date-validator.md) - A validator specifying the constraints on a date.
- [`Duration`](./duration.md) - A value that represents the difference between two timestamps.
- [`DurationValidator`](./duration-validator.md) - A validator specifying the constraints on a duration.
- [`EnumValidator`](./enum-validator.md) - A validator for a fixed set of allowed values.
- [`Hint`](./hint.md) - Union type for hints of the value and/or structure of data.
- [`Integer`](./integer.md) - An integer value.
- [`IntegerValidator`](./integer-validator.md) - A validator for integer values.
- [`Null`](./null.md) - The null value.
- [`Number`](./number.md) - A value that is a number.
- [`NumberValidator`](./number-validator.md) - A validator for numeric values.
- [`Object`](./object.md) - An object value.
- [`ObjectHint`](./object-hint.md) - A concise summary of the structure of an `Object`.
- [`Primitive`](./primitive.md) - A union type for primitive values.
- [`String`](./string.md) - A value comprised of a string of characters.
- [`StringHint`](./string-hint.md) - A concise summary of the properties of a `String`.
- [`StringValidator`](./string-validator.md) - A validator for string values.
- [`Time`](./time.md) - A point in time recurring on multiple days.
- [`TimeUnit`](./time-unit.md) - A unit in which time can be measured.
- [`TimeValidator`](./time-validator.md) - A validator specifying the constraints on a time.
- [`Timestamp`](./timestamp.md) - A value that represents a point in time.
- [`TimestampValidator`](./timestamp-validator.md) - A validator specifying the constraints on a timestamp.
- [`TupleValidator`](./tuple-validator.md) - A validator specifying constraints on an array of heterogeneous items.
- [`Unknown`](./unknown.md) - A placeholder for a value of unknown type.
- [`UnsignedInteger`](./unsigned-integer.md) - An integer value that is greater or equal to zero.
- [`Validator`](./validator.md) - A union type for validators.

# Flow

- [`Action`](./action.md) - An action performed by an agent.
- [`ActionAgent`](./action-agent.md) - A human, organization, software application, or Stencila AI agent that performs, provides, or participates in an action.
- [`ActionStatusType`](./action-status-type.md) - The status of an action.
- [`Button`](./button.md) - An interactive button.
- [`CallArgument`](./call-argument.md) - An argument used when calling a document.
- [`CallBlock`](./call-block.md) - Call another document, optionally with arguments, and include its executed content.
- [`CodeLocation`](./code-location.md) - A location within source code.
- [`CompilationDigest`](./compilation-digest.md) - A digest of an executable node and its dependencies.
- [`ConvertAction`](./convert-action.md) - An action that converts a resource from one representation or format to another.
- [`CreateAction`](./create-action.md) - An action that creates a result.
- [`Executable`](./executable.md) - An abstract base type for executable nodes.
- [`ExecuteAction`](./execute-action.md) - An action that executes code, a prompt, a workflow, or another executable node.
- [`ExecutionBounds`](./execution-bounds.md) - A boundary for node execution.
- [`ExecutionMode`](./execution-mode.md) - The circumstances under which a node should be executed.
- [`ExecutionRequired`](./execution-required.md) - A reason why a node does or does not require execution.
- [`ExecutionStatus`](./execution-status.md) - The status of a node's most recent execution.
- [`ExecutionTag`](./execution-tag.md) - An execution-affecting tag on code.
- [`ForBlock`](./for-block.md) - A block that repeats content for each item in an array.
- [`Form`](./form.md) - A form for batched updates to document parameters.
- [`FormDeriveAction`](./form-derive-action.md) - An action for applying a derived form value.
- [`Function`](./function.md) - A function signature.
- [`Graph`](./graph.md) - A directed graph relating Stencila nodes, used for provenance, reactivity, and similar relationship graphs.
- [`GraphAction`](./graph-action.md) - An action associated with a graph edge.
- [`GraphEdge`](./graph-edge.md) - A directed edge in a graph.
- [`GraphEdgeKind`](./graph-edge-kind.md) - The kind of directed relationship represented by a graph edge.
- [`GraphEvidence`](./graph-evidence.md) - Evidence for a graph edge.
- [`GraphEvidenceConfidence`](./graph-evidence-confidence.md) - The confidence level for graph evidence.
- [`GraphEvidenceKind`](./graph-evidence-kind.md) - The kind of evidence supporting a graph edge.
- [`GraphNode`](./graph-node.md) - A node in a graph.
- [`IfBlock`](./if-block.md) - A block that conditionally shows and executes alternative content.
- [`IfBlockClause`](./if-block-clause.md) - A clause within an `IfBlock` node.
- [`IncludeBlock`](./include-block.md) - A block that includes content from an external source.
- [`Parameter`](./parameter.md) - A document parameter.
- [`Variable`](./variable.md) - A named value.
- [`Walkthrough`](./walkthrough.md) - An interactive walkthrough.
- [`WalkthroughStep`](./walkthrough-step.md) - A step in a `Walkthrough`.

# Style

- [`Page`](./page.md) - A page in a document.
- [`Styled`](./styled.md) - An abstract base type for styled content.
- [`StyledBlock`](./styled-block.md) - Styled block content.
- [`StyledInline`](./styled-inline.md) - Styled inline content.

# Edits

- [`Instruction`](./instruction.md) - An abstract base type for document editing instructions.
- [`InstructionBlock`](./instruction-block.md) - An instruction to edit some block content.
- [`InstructionInline`](./instruction-inline.md) - An instruction to edit some inline content.
- [`InstructionMessage`](./instruction-message.md) - A message within an `Instruction`.
- [`MessagePart`](./message-part.md) - A union type for message parts.
- [`PromptBlock`](./prompt-block.md) - A preview of a rendered prompt at a location in a document.
- [`Suggestion`](./suggestion.md) - Abstract base type for nodes that indicate a suggested change to content.
- [`SuggestionBlock`](./suggestion-block.md) - Abstract base type for nodes that indicate a suggested change to block content.
- [`SuggestionInline`](./suggestion-inline.md) - Abstract base type for nodes that indicate a suggested change to inline content.
- [`SuggestionStatus`](./suggestion-status.md) - A review status for a suggestion.
- [`SuggestionType`](./suggestion-type.md) - A category of suggested edit.

# Other

- [`Brand`](./brand.md) - A brand used by an organization or person for labeling a product, product group, or similar.
- [`ContactPoint`](./contact-point.md) - A contact point, usually within an organization.
- [`Entity`](./entity.md) - An abstract base type for compound nodes.
- [`Enumeration`](./enumeration.md) - Lists or enumerations, for example, a list of cuisines or music genres, etc.
- [`Excerpt`](./excerpt.md) - An excerpt from a `CreativeWork`.
- [`Grant`](./grant.md) - A grant, typically financial or otherwise quantifiable, of resources.
- [`Island`](./island.md) - An island of content in a document.
- [`Labelled`](./labelled.md) - Abstract base for document nodes with labels.
- [`MessageLevel`](./message-level.md) - A severity level for a message.
- [`MessageRole`](./message-role.md) - A role in a message exchange.
- [`ModelParameters`](./model-parameters.md) - Model selection and inference parameters for generative AI models.
- [`MonetaryGrant`](./monetary-grant.md) - A monetary grant.
- [`Node`](./node.md) - Union type for all types in this schema, including primitives and entities
- [`Organization`](./organization.md) - An organization such as a school, NGO, corporation, club, etc.
- [`Person`](./person.md) - A person (alive, dead, undead, or fictional).
- [`PostalAddress`](./postal-address.md) - A physical mailing address.
- [`Product`](./product.md) - A product or service.
- [`PropertyValue`](./property-value.md) - A property-value pair.
- [`ProvenanceCategory`](./provenance-category.md) - A category of content provenance.
- [`ProvenanceCount`](./provenance-count.md) - The count of the number of characters in a `ProvenanceCategory` within an entity.
- [`RawBlock`](./raw-block.md) - A block of raw content in a specific format.
- [`RelativePosition`](./relative-position.md) - The position of one node relative to another.
- [`Role`](./role.md) - Represents additional information about a relationship or property.
- [`Thing`](./thing.md) - The most generic type of item.
- [`ThingVariant`](./thing-variant.md) - Union type for all types that are descended from `Thing`
