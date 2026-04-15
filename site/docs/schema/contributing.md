---
title: Contributing to Stencila Schema
description: Guidance for adding and updating schema types and related definitions.
---

Stencila Schema types and their properties are defined as JSON Schema, with a
small number of Stencila-specific extensions, but are authored in YAML files in
the `schema/` directory. This keeps the schema machine-readable and compatible
with JSON Schema concepts while making it easier to read, review, and edit by
hand.

In practice, this means each YAML file defines a schema type using familiar JSON
Schema constructs such as `type`, `description`, `required`, `properties`, and
`anyOf`, alongside Stencila-specific fields used for code generation,
documentation, serialization, and runtime behavior.

> [!tip] Tips
>
> - For readability in YAML source, wrap prose such as `description` and root
>   `$comment` text at around 80 columns, including indentation.

# description

Use the root `description` field to say plainly and briefly what the type is.
Unlike the root [`$comment`](#comment), the `description` should be a short,
definition-style summary rather than an explanation of provenance, rationale, or
Stencila-specific extensions.

For most types, prefer a single sentence that names the thing directly,
describes the type itself rather than its full behavior, and matches the source
vocabulary when appropriate.

> [!tip] Tips
>
> - Keep `description` fields short, usually one sentence.
> - Prefer defining what the type is, not why it exists in Stencila Schema.
> - Do not duplicate the root `$comment`. Put rationale, origin, and
>   Stencila-specific semantics in `$comment` instead.
> - Avoid listing key properties in `description`; reserve that for `$comment`
>   when needed.
> - Prefer neutral, definition-style wording over explanatory prose.
> - Prefer “A …”, “An …”, or “The …” openings.
> - For schema.org-derived types, prefer a close adaptation of the source
>   definition unless Stencila meaningfully narrows or broadens it.
> - For Stencila-native types, use a concise definition that helps readers
>   recognize the node's role in a document or data model.

# $comment

Use root `$comment` fields to explain why a type exists in Stencila Schema, especially when it is an implementation, adaptation, or extension of a type from schema.org or another external specification. The goal is not to restate the `description` or list every property, but to explain the rationale for the type and how Stencila uses, extends, or adapts it.

For most concrete types, prefer a short multi-paragraph structure covering the
type's origin and relationship to other vocabularies, the Stencila-specific
semantics it adds or emphasizes, and the small number of properties that carry
the main Stencila-specific meaning.

> [!tip] Tips
> 
> - Keep root `$comment` fields conceptual. Put constraints, edge cases, and
>   format-specific notes on the relevant properties instead.
> - Do not duplicate the `description` field. The `description` should say what
>   the type is; the root `$comment` should say why it is in Stencila Schema and
>   what Stencila changes, adds, or emphasizes.
> - For inherited behavior, refer to the parent type rather than repeating its full semantics.
> - State whether the type is an implementation of a schema.org type, an
>   extension of one, a renaming of one, or a Stencila-native type with
>   analogues elsewhere.
> - For schema.org-derived types, prefer to say so in the first sentence and
>   link to the source type.
> - Explain what Stencila adds, changes, or constrains beyond the source
>   vocabulary or external specification.
> - Point readers to the 2–5 properties that carry the main Stencila-specific
>   semantics, rather than trying to summarize the full schema.
> - Prefer a consistent opening for schema.org-derived types, for example:
>   “This is an implementation of schema.org
>   [`Type`](https://schema.org/Type).”, “This is an implementation of
>   schema.org [`Type`](https://schema.org/Type), extended in Stencila Schema to
>   support …”, or “This is an implementation of schema.org
>   [`Type`](https://schema.org/Type), exposed in Stencila Schema as `LocalName`
>   to …”.
> - For Stencila-native types, explain the closest analogues only when that helps readers orient themselves.
> - Avoid exhaustive lists of related standards or properties unless they
>   materially help explain the type.
> - Prefer 1–3 concise paragraphs. Foundational or complex types may need more,
>   but brevity and consistency are preferred.
> - As a rule of thumb, a good root `$comment` should help a reader answer three
>   questions quickly: where does this type come from, what does Stencila add,
>   change, or constrain, and which properties should I look at next?
