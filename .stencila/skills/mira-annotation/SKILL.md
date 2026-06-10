---
name: mira-annotation
description: Annotate Markdown documents with MIRA research objects and source-local relations. Use for md, smd, qmd, and myst files when identifying claims, evidence, questions, protocols, requests, typed claims, and relation attributes while preserving the original Markdown flavor and author wording. Not for general copyediting, prose rewriting, or extracting a separate knowledge graph without annotating the source.
keywords:
  - mira
  - markdown
  - annotation
  - semantic annotation
  - research objects
  - claim
  - evidence
  - question
  - protocol
  - request
  - relations
  - supports
  - supported-by
  - md
  - smd
  - qmd
  - myst
allowed-tools: read_file write_file apply_patch glob grep ask_user
---

## Overview

Annotate Markdown documents with MIRA research objects and relations. Identify claims, evidence, questions, protocols, requests, and related scholarly objects in the source text, then wrap or mark those passages using syntax that is idiomatic for the document's Markdown flavor.

This skill is for semantic research-object annotation, not copyediting. Preserve the author's wording and document structure unless the user explicitly asks for rewriting.

## Required Inputs

| Input | Required | Description |
|---|---|---|
| Target Markdown content or file path | Required | The Markdown text or document to annotate |
| Markdown flavor or file extension | Optional | One of `.md`, `.smd`, `.qmd`, or `.myst`; infer from the file path and local syntax when absent |
| Annotation scope | Optional | Whether to annotate the entire document, a section, or only selected passages |
| Object vocabulary preferences | Optional | Any user or project preference for specific object types, typed claims, ids, or relation style |
| Delivery mode | Optional | Whether to edit the source file, return annotated Markdown, or provide an annotation plan first |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output | Description |
|---|---|
| Annotated Markdown | The source content with MIRA research-object wrappers, directive options, or attributes added in the appropriate Markdown dialect |
| Summary of annotations | A concise note listing major object types and relations added or intentionally left uncertain |
| Assumptions or questions | Any assumptions made about ambiguous syntax, relation direction, or object category; ask before editing when ambiguity materially affects the result |
| Validation status | Confirmation that fences/directives are balanced, ids are unique, relation targets exist, and the result remains valid for the detected flavor |

## Core Behavior

1. Detect the Markdown flavor from the file extension and local syntax:
   - `.smd`: Stencila Markdown fences such as `::: claim #claim-id` and `::: evidence #evidence-id`.
   - `.qmd`: Quarto fenced divs such as `::: {.claim #claim-id}` or `::: {.statement #claim-id}`.
   - `.myst`: MyST directives such as ```` ```{claim} Title```` or ```` ```{evidence} Title```` with option lines like `:id: evidence-id`.
   - `.md`: prefer the least disruptive syntax already present in the file; if no project convention is evident, use Stencila Markdown-style research object fences because they are readable in plain Markdown and decode to Stencila research objects.
2. Preserve the author's wording unless the user explicitly asks for rewriting. Wrap or mark existing text; do not change scientific meaning to make it fit an annotation.
3. Create stable, descriptive, unique, kebab-case ids for each research object. Prefix ids by type when useful, such as `claim-dose-response`, `evidence-cohort-study`, or `question-mechanism`.
4. Annotate at the smallest useful block granularity. Avoid over-annotating every sentence; combine adjacent sentences when they express one coherent claim, evidence item, question, protocol, or request.
5. Prefer relations that are explicitly signaled by the document. Use inferred relations only when the connection is strong and local; otherwise leave the relation unannotated or mention uncertainty in the response.
6. Preserve existing frontmatter, headings, citations, cross-references, code cells, math, tables, lists, comments, and custom directives. Do not break executable code chunks, notebook metadata, Quarto options, or MyST directives.
7. If the file already contains research-object annotations, extend and normalize them rather than duplicating objects. Reuse existing ids, object names, and relation style where possible.
8. Validate before finishing: check that fences and directives are balanced, ids are unique, relations point to existing ids, and the output remains valid for the detected Markdown flavor.
9. If the document flavor, desired object vocabulary, relation direction, or relation encoding is ambiguous, ask concise clarifying questions before editing. Otherwise make the smallest safe annotation pass and explain the assumptions used.

## MIRA Object Categories

Use these categories consistently:

- `claim`: an assertion, conclusion, theorem, proposition, hypothesis, statement, or interpretation that can be supported or opposed.
- Typed claims: use a more specific type such as `hypothesis`, `statement`, `proposition`, `theorem`, `lemma`, `proof`, `corollary`, or `postulate` when the text clearly matches that type or the document already uses that convention.
- `evidence`: observations, measurements, citations, results, examples, excerpts, datasets, or summaries used to support or oppose a claim.
- `question`: explicit research questions, open problems, or unresolved issues.
- `protocol`: methods, procedures, experimental designs, computational workflows, or analysis plans.
- `request`: authored requests for evidence, work, protocol execution, review, contribution, or clarification.

Avoid inventing objects that are not grounded in the source. If a sentence is only background context and is not functioning as a claim, evidence, question, protocol, or request, leave it unannotated.

## Relation Vocabulary and Direction

Identify source-local relations between objects and encode them as attributes or directive options on the source object. Supported relation names are:

- `supports` / `supported-by`
- `opposes` / `opposed-by`
- `addresses` / `addressed-by`
- `follows`
- `grounds` / `is-grounded-in`
- `request-for`
- `request-target`

Guidelines:

1. Use relation direction deliberately. For example, a claim may use `supported-by` to point to evidence; evidence may use `supports` to point to a claim. Match any existing relation style in the document before introducing the inverse form.
2. Encode relations to object ids, usually with a leading `#` such as `#evidence-study`. Do not point a relation at a citation key or heading unless a corresponding research object id exists.
3. Prefer one clear relation over multiple weak ones. Do not add relation attributes simply because two objects occur near each other.
4. When multiple relations are needed, follow the existing file convention if present. Otherwise keep the encoding simple and readable for the dialect.

## Dialect Syntax Guide

### Stencila Markdown (`.smd`, and default for plain `.md` when no other convention is present)

Use colon fences with the object type and optional id. Add relation options as colon-prefixed lines inside the object block before the content.

```markdown
::: claim #claim-main
:supported-by: #evidence-study

The intervention reduces symptom severity in adults with condition X.
:::

::: evidence #evidence-study

In a randomized trial, the intervention group showed a larger reduction in symptom scores than controls.
:::
```

For plain `.md`, first look for an existing convention such as HTML comments, fenced divs, or Stencila-style fences. If no convention is evident, prefer the Stencila Markdown style above because it remains readable in plain Markdown.

### Quarto Markdown (`.qmd`)

Use Quarto fenced divs with classes, ids, and relation attributes on the opening fence.

```markdown
::: {.claim #claim-main supported-by="#evidence-study"}

The intervention reduces symptom severity in adults with condition X.
:::

::: {.evidence #evidence-study}

In a randomized trial, the intervention group showed a larger reduction in symptom scores than controls.
:::
```

Preserve Quarto YAML frontmatter, chunk options, callouts, cross-references, and executable code cells. Do not move or reformat code chunks merely to add annotations.

### MyST Markdown (`.myst`)

Use MyST directives with `:id:` and relation option lines.

````markdown
```{claim} Main effect
:id: claim-main
:supported-by: #evidence-study

The intervention reduces symptom severity in adults with condition X.
```

```{evidence} Randomized trial
:id: evidence-study

In a randomized trial, the intervention group showed a larger reduction in symptom scores than controls.
```
````

For MyST proof-style claims, use `prf:` directives when the document already uses the proof extension or the claim type is mathematical, for example ```` ```{prf:theorem} ```` or ```` ```{prf:statement} ````. Preserve existing MyST roles, directives, labels, and option syntax.

## Annotation Workflow

1. Read the target content before editing and identify the file extension, existing annotation syntax, ids, relation style, and Markdown flavor conventions.
2. Scan the requested scope for candidate research objects:
   - assertions or conclusions that should become claims or typed claims;
   - observations, results, citations, examples, or measurements that function as evidence;
   - explicit research questions or unresolved issues;
   - methods, procedures, or analysis plans that function as protocols;
   - requests for work, evidence, review, protocol execution, or contribution.
3. Choose object boundaries. Prefer coherent blocks over sentence-by-sentence wrapping. Preserve lists, blockquotes, tables, and math as intact as possible.
4. Assign ids. Reuse existing ids where possible; otherwise create stable, descriptive, kebab-case ids and ensure uniqueness across the document.
5. Add object wrappers or directive syntax matching the detected dialect. Keep original text inside the wrapper unchanged except for indentation needed by the dialect.
6. Add relations only when they are explicit or strongly local. Use existing relation direction/style where present.
7. Re-read the annotated result for preservation issues: frontmatter, citations, code cells, math, headings, cross-references, lists, and custom directives should remain intact.
8. Check all ids and relation targets. Every relation target should correspond to an annotated object id in the same document unless the user or project convention clearly allows cross-document targets.
9. Report a concise summary of changes, assumptions, and any relations left uncertain.

## Preservation Rules

- Do not broadly copyedit, simplify, translate, or rewrite prose as part of annotation.
- Do not change numerical values, units, citations, equations, code, output, or scientific meaning.
- Do not annotate inside executable code blocks, raw HTML blocks, or generated output unless the user explicitly requests it and the format supports the change safely.
- Do not split Markdown constructs in ways that alter rendering, such as breaking a table, list item, admonition, Quarto callout, MyST directive, or code chunk.
- Do not remove existing annotations because they use a different but valid style; normalize only when needed to avoid duplicates or resolve inconsistent ids.
- Do not add speculative relations across distant sections unless the document explicitly states the connection.

## Examples

### Example 1: Stencila Markdown claim supported by evidence

Input passage:

```markdown
The intervention reduces symptom severity in adults with condition X.

In a randomized trial, the intervention group showed a larger reduction in symptom scores than controls.
```

Annotated output:

```markdown
::: claim #claim-intervention-reduces-symptoms
:supported-by: #evidence-randomized-trial

The intervention reduces symptom severity in adults with condition X.
:::

::: evidence #evidence-randomized-trial

In a randomized trial, the intervention group showed a larger reduction in symptom scores than controls.
:::
```

### Example 2: Quarto question addressed by a protocol

```markdown
::: {.question #question-mechanism addressed-by="#protocol-mediation-analysis"}

Which pathway explains the observed association between exposure and outcome?
:::

::: {.protocol #protocol-mediation-analysis}

We estimate direct and indirect effects using a preregistered mediation model adjusted for baseline covariates.
:::
```

### Example 3: MyST mathematical theorem using proof-style directives

````markdown
```{prf:theorem} Monotonicity
:id: theorem-monotonicity

For every positive value of x, the transformation is monotone increasing.
```

```{proof} Monotonicity proof
:id: proof-monotonicity
:grounds: #theorem-monotonicity

The derivative is positive on the stated domain, so the transformation is monotone increasing.
```
````

Use `prf:` directives only when the document already uses MyST proof syntax or the claim is clearly mathematical.

## Edge Cases

- **Unknown extension**: infer from local syntax. If no convention is clear, ask the user whether to use Stencila Markdown-style fences before making a large edit.
- **Mixed conventions**: follow the dominant local convention in the edited section. Avoid mixing dialects within one document unless the existing file already does so.
- **Existing duplicate ids**: do not create more duplicates. Reuse the intended object id when clear, or choose a new descriptive id and mention the duplicate in the summary.
- **Ambiguous relation direction**: omit the relation or ask a clarification question rather than adding a misleading inverse relation.
- **Large documents**: annotate the requested scope first. If no scope is given, make a focused first pass on the most clearly signaled objects and summarize remaining opportunities.
- **Only a knowledge graph requested**: this skill is for annotating source Markdown. If the user explicitly wants a separate graph without source annotations, explain that this is outside the skill's focus and ask whether they want source annotations instead.
