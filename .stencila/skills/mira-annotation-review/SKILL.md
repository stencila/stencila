---
name: mira-annotation-review
description: Review MIRA annotations in Markdown documents for semantic correctness, Markdown dialect validity, preservation, and relation integrity. Use for md, smd, qmd, and myst files containing research objects such as claims, evidence, questions, protocols, requests, typed claims, ids, and relations. Produces actionable findings without modifying files by default.
keywords:
  - mira
  - annotation review
  - markdown review
  - semantic correctness
  - research objects
  - relation integrity
  - preservation
  - dialect validity
  - claim
  - evidence
  - question
  - protocol
  - request
  - supports
  - supported-by
  - md
  - smd
  - qmd
  - myst
  - quarto
  - stencila markdown
  - myst markdown
  - no-edit review
allowed-tools: read_file glob grep shell ask_user
---

## Overview

Review MIRA annotations in Markdown documents without changing the files unless the user explicitly asks for edits. Check that annotated research objects and relations are semantically grounded in the source text, valid for the document's Markdown dialect, internally consistent, and preservative of the original document structure and meaning.

This skill is for annotation quality assurance, not for general prose editing or adding new annotations. Prefer concrete, actionable findings over broad advice.

## Required Inputs

| Input | Required | Description |
|---|---|---|
| Target Markdown content or file path | Required | The annotated Markdown text or one or more `.md`, `.smd`, `.qmd`, or `.myst` files to review |
| Markdown flavor or file extension | Optional | One of Stencila Markdown, Quarto Markdown, MyST Markdown, or plain Markdown; infer from path and syntax when absent |
| Review scope | Optional | Entire document, selected sections, specific annotation ids, or a particular concern such as relation integrity |
| MIRA vocabulary or project conventions | Optional | Any project-specific allowed object types, relation names, id style, or dialect conventions |
| Desired output detail | Optional | Summary only, full finding table, line-referenced findings, or prioritized remediation plan |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output | Description |
|---|---|
| Review summary | Overall assessment of annotation quality, main risks, and whether the annotations appear usable |
| Findings | Specific issues with severity, location, affected ids or text, explanation, and recommended fix |
| Validation notes | Dialect syntax, id uniqueness, relation target, and preservation checks that passed or could not be fully checked |
| Open questions | Ambiguities requiring author or project clarification before changing annotations |

## Review Principles

1. Do not modify source files by default. If the user asks for fixes, first provide findings or confirm the intended edits when changes could alter scientific meaning.
2. Treat source text as authoritative. An annotation is valid only when it is supported by the words, structure, or explicitly stated relationships in the document.
3. Preserve document semantics. Do not recommend changes that would alter numerical values, citations, equations, code behavior, frontmatter, headings, cross-references, or scientific claims.
4. Prefer the document's existing annotation style when it is valid. Do not flag harmless style variation unless it causes ambiguity, parser failure, or inconsistent interpretation.
5. Distinguish errors from uncertainty. Mark speculative findings as questions or warnings rather than inventing corrections.
6. Focus on actionable review outcomes: what is wrong, where it occurs, why it matters, and how to fix it.

## MIRA Objects to Recognize

Review these object categories and any project-supplied extensions:

- `claim`: an assertion, conclusion, interpretation, hypothesis, theorem-like statement, or proposition that can be supported or opposed.
- Typed claims: more specific claim-like objects such as `hypothesis`, `statement`, `proposition`, `theorem`, `lemma`, `proof`, `corollary`, or `postulate` when the source text or project convention clearly supports that type.
- `evidence`: observations, measurements, results, examples, citations, datasets, excerpts, or summaries used to support or oppose a claim.
- `question`: explicit research questions, open problems, or unresolved issues.
- `protocol`: methods, procedures, experimental designs, computational workflows, or analysis plans.
- `request`: authored requests for evidence, work, protocol execution, review, contribution, clarification, or other action.

Flag annotations that create research objects from text that is only navigational, decorative, boilerplate, generic background, or too vague to support the assigned type.

## Relation Vocabulary and Direction Checks

Common relation names include:

- `supports` / `supported-by`
- `opposes` / `opposed-by`
- `addresses` / `addressed-by`
- `follows`
- `grounds` / `is-grounded-in`
- `request-for`
- `request-target`

Check each relation for:

1. **Grounding**: the document explicitly states or strongly locally implies the relation.
2. **Direction**: the source and target roles match the relation name. For example, evidence `supports` a claim, while a claim is `supported-by` evidence.
3. **Target existence**: relation targets refer to an annotated object id in the document unless cross-document targets are a declared convention.
4. **Target type**: the target object type is plausible for the relation. For example, `supported-by` should usually point to evidence, not to an unrelated heading or code chunk.
5. **Encoding consistency**: relation values consistently use ids, usually with `#id`, and follow the document's existing style.
6. **Non-speculation**: relations are not added merely because objects are adjacent or topically similar.
7. **Inverse consistency**: if both inverse directions are present, they should agree rather than conflict or duplicate conflicting meanings.

## Dialect Review Guide

### Stencila Markdown (`.smd`, and often plain `.md`)

Typical object blocks use colon fences with an object type and optional id. Relation options may appear as colon-prefixed lines inside the block.

```markdown
::: claim #claim-main
:supported-by: #evidence-study

The intervention reduces symptom severity.
:::
```

Review checks:

1. Opening and closing colon fences are balanced and nested safely.
2. The object type appears immediately after the opening fence.
3. Ids are stable, unique, and valid for the local convention, typically kebab-case with a leading `#` in the fence.
4. Option lines use the expected `:name: value` form and appear where the dialect accepts them.
5. The wrapper does not split tables, lists, blockquotes, math blocks, code fences, or other Markdown constructs incorrectly.

### Quarto Markdown (`.qmd`)

Typical object blocks use fenced div attributes with classes, ids, and relation attributes on the opening fence.

```markdown
::: {.claim #claim-main supported-by="#evidence-study"}

The intervention reduces symptom severity.
:::
```

Review checks:

1. Fenced div attributes are enclosed in `{...}` and use valid class, id, and key-value syntax.
2. Object type classes such as `.claim` or `.evidence` are present and not confused with Quarto callout classes unless that is intentional.
3. Relation attributes are quoted when needed and do not break YAML-like or Pandoc attribute parsing.
4. Quarto YAML frontmatter, executable code cells, chunk options, callouts, figures, tables, and cross-references remain intact.
5. Annotation fences do not wrap code chunks or divs in a way that changes Quarto execution or rendering.

### MyST Markdown (`.myst`)

Typical object blocks use directives with option lines such as `:id:` and relation options.

````markdown
```{claim} Main effect
:id: claim-main
:supported-by: #evidence-study

The intervention reduces symptom severity.
```
````

Review checks:

1. Directive fences are balanced and use valid directive names.
2. Option lines use MyST directive option syntax and appear before body content.
3. Ids are represented consistently, commonly without `#` in `:id:` and with `#` when referenced.
4. MyST roles, substitutions, labels, admonitions, proof directives, and cross-references are preserved.
5. Proof-style mathematical objects use project-appropriate directive names such as `prf:theorem` when that convention is already present.

### Plain Markdown (`.md`)

Plain Markdown may contain Stencila-style fences, Quarto/Pandoc fenced divs, HTML comments, or project-specific conventions. Infer the intended style from existing annotations before flagging syntax. If no convention is clear, report ambiguity rather than assuming one dialect is mandatory.

## Review Workflow

1. Identify the target files or content and infer the Markdown flavor from extension, frontmatter, existing fences/directives, code cells, and cross-reference syntax.
2. Read the annotated content carefully before making judgments. If reviewing multiple files, first list them and note mixed dialects or shared id conventions.
3. Inventory all MIRA objects: object type, id, location, boundary text, relation attributes, and nested or surrounding Markdown constructs.
4. Check semantic correctness:
   - Does each object type match the role of the annotated text?
   - Are typed claims used only where the source clearly warrants the subtype?
   - Are questions, protocols, requests, and evidence distinguished correctly?
5. Check object boundaries:
   - Is the annotation neither too narrow nor too broad for the research object?
   - Does it preserve paragraphs, list items, tables, blockquotes, math, figures, citations, and code fences?
   - Does it avoid wrapping unrelated context or splitting a coherent object into unnecessary fragments?
6. Check ids:
   - Each annotated object that is referenced has an id.
   - Ids are unique within the review scope.
   - Ids are stable, readable, and consistent with existing naming conventions.
   - Renaming suggestions account for all relation targets and cross-references.
7. Check relation integrity using the relation vocabulary and direction checks above.
8. Check dialect validity using the dialect review guide. When tools are available and appropriate, run non-destructive validation commands such as Markdown parsing or conversion; report tool limitations rather than treating missing tools as annotation failures.
9. Check preservation:
   - Frontmatter, citations, footnotes, cross-references, code chunks, outputs, math, tables, lists, comments, custom directives, and raw HTML remain semantically unchanged.
   - Annotation wrappers do not change execution behavior or rendering in Stencila, Quarto, or MyST.
10. Detect over-annotation and duplication:
    - Avoid flagging valid dense annotation when the scope calls for it, but report annotations that mark every sentence without a research-object purpose.
    - Report duplicate objects, repeated ids, contradictory object types for the same text, and inconsistent relation encodings.
11. Produce findings with severity and actionable fixes. If a problem cannot be confirmed, label it as an open question.

## Finding Severity

Use these severities consistently:

- **Critical**: likely parser failure, invalid nesting that breaks the document, missing relation targets required for interpretation, or annotation changes scientific meaning.
- **Major**: semantically wrong object type, reversed relation direction, duplicate ids, unsupported relation, or boundary that substantially changes meaning.
- **Minor**: style inconsistency, unclear id naming, low-risk over-annotation, or relation encoding variation that is understandable but should be normalized.
- **Question**: ambiguous issue needing author, project, or dialect clarification.

## Recommended Finding Format

Use a table when possible:

| Severity | Location | Object or relation | Issue | Recommendation |
|---|---|---|---|---|
| Major | `file.smd`, section "Results" | `claim-main supported-by #claim-main` | The claim points to itself as supporting evidence | Point `supported-by` to the evidence object id or remove the relation |

If line numbers are unavailable, cite the nearest heading, object id, and a short quoted excerpt.

## Review Checklist

### Semantic Correctness

- Object type matches the text's rhetorical and research role.
- Evidence objects contain evidence, not unsupported interpretations.
- Claim objects contain claims, not mere section summaries or citations alone.
- Questions are explicit or clearly unresolved, not statements mislabeled as questions.
- Protocols describe methods or procedures, not results.
- Requests ask for action, information, execution, contribution, or review.
- Typed claims are justified by the text and local convention.

### Object Boundaries

- Boundaries include the whole object and exclude unrelated context.
- Adjacent sentences are grouped only when they express one coherent object.
- Lists, tables, blockquotes, callouts, directives, and code fences are not broken.
- Nested annotations are valid for the dialect and do not create ambiguous ownership.

### Ids and References

- Ids are unique and stable in the review scope.
- Referenced objects have ids.
- Id formats are consistent, typically descriptive kebab-case.
- Relation targets and cross-references use the correct `#id` or bare id form for their context.
- Duplicate ids, near-duplicate ids, and stale references are reported.

### Relations

- Relation names are recognized or project-approved.
- Direction matches object roles.
- Targets exist and have plausible types.
- Inverse relations do not contradict each other.
- Relations are grounded in text, not speculative.
- Cross-document relations are allowed only when the project convention says so.

### Dialect Validity

- Fences/directives/divs are balanced.
- Attribute and option syntax matches Stencila Markdown, Quarto, MyST, or the declared project convention.
- Frontmatter and executable cell syntax are preserved.
- Annotation syntax is not mixed in a way that breaks parsing.
- Custom directives or extensions are treated carefully and not assumed invalid solely because they are unfamiliar.

### Preservation

- Original wording, values, units, citations, equations, and code are unchanged.
- Rendering-sensitive structures retain their shape.
- Generated outputs and notebook metadata are not annotated or edited unless explicitly requested.
- Existing valid annotations are not rejected simply because another style is preferred.

## Examples

### Example 1: Reversed Relation Direction

Annotated passage:

```markdown
::: claim #claim-treatment-works
:supports: #evidence-trial

The treatment reduces symptom severity.
:::

::: evidence #evidence-trial

A randomized trial found lower symptom scores in the treatment group.
:::
```

Finding:

| Severity | Location | Object or relation | Issue | Recommendation |
|---|---|---|---|---|
| Major | `#claim-treatment-works` | `supports #evidence-trial` | The relation direction is reversed: the claim is supported by the evidence; it does not support the evidence. | Change to `:supported-by: #evidence-trial` on the claim, or place `:supports: #claim-treatment-works` on the evidence. |

### Example 2: Over-Broad Boundary

Annotated passage:

```markdown
::: evidence #evidence-background-and-result
The disease is common worldwide. In the trial, 62% of treated participants improved compared with 41% of controls. Future work should evaluate durability.
:::
```

Finding:

| Severity | Location | Object or relation | Issue | Recommendation |
|---|---|---|---|---|
| Major | `#evidence-background-and-result` | Object boundary | The evidence annotation includes background context and a future-work request in addition to the trial result. | Restrict the evidence object to the trial result sentence and consider a separate request only if future work should be annotated. |

### Example 3: MyST Id Reference Form

Annotated passage:

````markdown
```{evidence} Trial result
:id: #evidence-trial

A randomized trial found lower symptom scores.
```
````

Finding:

| Severity | Location | Object or relation | Issue | Recommendation |
|---|---|---|---|---|
| Minor | `Trial result` directive | `:id: #evidence-trial` | MyST ids are commonly declared without the leading `#`; the leading marker is typically used when referencing the id. | Use `:id: evidence-trial` unless the project convention explicitly requires `#` in declarations. |

## Edge Cases

- **Mixed dialect files**: If a document intentionally combines dialect features, check whether the combination is accepted by the target processor before flagging it. Report uncertainty when the intended processor is unknown.
- **Existing invalid annotations**: Review and report them; do not silently normalize or remove them unless the user requests fixes.
- **Cross-document relation targets**: Treat missing local targets as findings unless the user or project convention permits cross-document ids. Ask for clarification when likely intentional.
- **Generated or executable content**: Do not review code output, generated cells, or machine-generated sections as authorial annotations unless explicitly in scope.
- **Ambiguous object type**: Prefer a question or minor/major finding that explains the ambiguity over confidently relabeling the object.
- **Unrecognized relation names**: Check whether they are project-specific before marking them invalid. If no convention is supplied, report them as questions or minor/major findings depending on impact.
- **No annotations found**: Report that there are no MIRA annotations to review and, if useful, identify candidate locations only as optional observations rather than required fixes.
- **User asks for direct edits**: Confirm any semantically risky changes first, then preserve style and update all impacted ids and relation targets consistently.

## Validation Commands

Use commands only when they are available and relevant, and do not let a missing tool block a useful manual review. Examples of non-destructive checks include:

```sh
stencila convert document.smd --to json
quarto render document.qmd --to html --quiet
myst build document.myst
```

Report the command, result, and any limitations. Avoid commands that overwrite source files or generated outputs unless the user explicitly approves.
