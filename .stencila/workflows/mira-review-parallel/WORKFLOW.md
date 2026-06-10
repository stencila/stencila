---
name: mira-review-parallel
title: MIRA Review Parallel Workflow
description: Run three independent MIRA annotation reviews in parallel using Anthropic, OpenAI, and Google models via the `mira-reviewer` agent, then synthesize a unified prioritized findings report highlighting reviewer agreement and disagreement
goal-hint: What MIRA-annotated Markdown document should be reviewed? (e.g., a file path or description of the annotation target)
keywords:
  - mira
  - annotation review
  - markdown
  - research objects
  - semantic annotation
  - parallel
  - multi-model
  - consensus
  - disagreement
  - prioritized findings
when-to-use:
  - when you want independent reviews of MIRA annotations from multiple LLM families to reduce blind spots
  - when you want a synthesized report that highlights agreement and disagreement about research-object types, ids, relations, and Markdown dialect syntax
  - when checking annotations created by mira-annotator, another workflow, or a human author
when-not-to-use:
  - when MIRA annotations need to be added to an unannotated document rather than reviewed
  - when a single-model MIRA annotation review is sufficient
  - when the task is general copyediting, prose rewriting, or non-MIRA document review
---

This workflow fans out to three parallel MIRA annotation reviews of the same Markdown target, each using the `mira-reviewer` agent but forced to a different model provider (Anthropic, OpenAI, Google). After all three complete, a synthesis node merges the reviews into a single numbered, prioritized findings list that calls out agreement and disagreement across reviewers. The workflow ends immediately after synthesis so it can be composed by parent workflows without an extra human gate; callers that need human approval should add their own review step after this workflow.

```dot
digraph mira_review_parallel {
  Start -> FanOutReviews

  FanOutReviews [label="Review MIRA annotations in parallel with three model families", timeout="15m"]
  FanOutReviews -> ReviewA
  FanOutReviews -> ReviewB
  FanOutReviews -> ReviewC

  ReviewA [agent="mira-reviewer", agent.provider="anthropic", prompt="Review the MIRA annotations in the following Markdown target: $goal"]
  ReviewA -> Synthesize

  ReviewB [agent="mira-reviewer", agent.provider="openai", prompt="Review the MIRA annotations in the following Markdown target: $goal"]
  ReviewB -> Synthesize

  ReviewC [agent="mira-reviewer", agent.provider="google", prompt="Review the MIRA annotations in the following Markdown target: $goal"]
  ReviewC -> Synthesize

  Synthesize [prompt-ref="#synthesize-prompt"]
  Synthesize -> End
}
```

```markdown #synthesize-prompt
You have received three independent MIRA annotation reviews of the same Markdown target. Retrieve the output of each review node using the available workflow tools.

Produce a single unified report with these sections:

1. **Summary** — a brief overview of the document or annotation target under review and the overall assessment of annotation quality.

2. **Prioritized Findings** — a numbered list of findings ordered by severity (critical → high → medium → low → informational). For each finding:
   - **Title**: concise name
   - **Severity**: critical / high / medium / low / informational
   - **Agreement**: note which reviewers flagged this (e.g., "All three", "A + B", "C only")
   - **Location**: file, heading, line, id, or other locator if available
   - **Description**: what the annotation issue is and why it matters
   - **Recommendation**: concrete fix or improvement

3. **Consensus & Disagreement** — highlight:
   - Areas where all three reviewers agreed
   - Areas where reviewers disagreed or only one reviewer flagged an issue, with brief reasoning about which perspective seems strongest

4. **MIRA-Specific Checks** — summarize the state of:
   - Research-object type choices (claims, evidence, questions, protocols, requests, and any other MIRA object types used)
   - Identifier uniqueness, stability, and readability
   - Relation correctness and directionality
   - Markdown dialect syntax for the target format (md, smd, qmd, or myst)
   - Preservation of author content and avoidance of unintended prose edits

5. **Overall Recommendation** — a single clear recommendation (approve, approve with changes, or request revisions) based on the weight of evidence across all three reviews.

Before finalizing, validate each finding against the actual Markdown document when possible. Pay particular attention to findings flagged by only one reviewer — these are the most likely to be either a genuine insight the others missed or a hallucinated issue. Read the relevant Markdown to confirm the finding is real. Downgrade or drop any finding you cannot substantiate, and note when a single-reviewer finding was confirmed or rejected by your own inspection.

Be concise. Do not reproduce the full text of each review — synthesize and deduplicate.

Write the final report to `.stencila/reviews/` as a Markdown file. Derive the filename from the review target using kebab-case and include `mira-review` in the name (e.g., `.stencila/reviews/article-mira-review.md`). Create the directory if it does not exist. If a file with that name already exists, choose another name.

Then output the same report in full as your final message. The final message is the canonical workflow output for composition by parent workflows. Do not respond with only a file path, success message, or summary. After the report, add a final line indicating where the copy was saved.
```
