---
title: "Code Review Parallel Workflow"
description: "Run three independent code reviews in parallel using Anthropic, OpenAI, and Google models via the `software-code-reviewer` agent, then synthesize a unified prioritized findings report highlighting reviewer agreement and disagreement"
keywords:
  - code review
  - parallel
  - multi-model
  - consensus
  - disagreement
  - prioritized findings
---

Run three independent code reviews in parallel using Anthropic, OpenAI, and Google models via the `software-code-reviewer` agent, then synthesize a unified prioritized findings report highlighting reviewer agreement and disagreement

**Keywords:** code review · parallel · multi-model · consensus · disagreement · prioritized findings

> [!tip] Usage
>
> To run this workflow, start your prompt with `~code-review-parallel` followed by your goal, or select it with the `/workflow` command.

# When to use

- when you want independent code reviews from multiple LLM families to reduce blind spots
- when you want a synthesized report that highlights where reviewers agree and disagree

# When not to use

- when a single-model code review is sufficient
- when the task is implementation, refactoring, or test writing rather than review

# Configuration

| Property | Value |
| -------- | ----- |
| Goal | What code should be reviewed? (e.g., a file path, package name, or description of the change) |
| Referenced agents | [`software-code-reviewer`](/docs/agents/builtin/software/software-code-reviewer/) |

# Pipeline

This workflow fans out to three parallel code reviews of the same target, each using the `software-code-reviewer` agent but forced to a different model provider (Anthropic, OpenAI, Google). After all three complete, a synthesis node merges the reviews into a single numbered, prioritized findings list that calls out agreement and disagreement across reviewers. A human review gate at the end lets you accept or send the synthesis back for revision.

```dot
digraph code_review_parallel {
  Start -> FanOutReviews

  FanOutReviews [label="Review in parallel with three model families"]
  FanOutReviews -> ReviewA
  FanOutReviews -> ReviewB
  FanOutReviews -> ReviewC

  ReviewA [agent="software-code-reviewer", agent.provider="anthropic", prompt="Review the following: $goal"]
  ReviewA -> Synthesize

  ReviewB [agent="software-code-reviewer", agent.provider="openai", prompt="Review the following: $goal"]
  ReviewB -> Synthesize

  ReviewC [agent="software-code-reviewer", agent.provider="google", prompt="Review the following: $goal"]
  ReviewC -> Synthesize

  Synthesize [prompt-ref="#synthesize-prompt"]
  Synthesize -> HumanReview

  HumanReview [interview-ref="#human-review"]
  HumanReview -> End     [label="Accept"]
  HumanReview -> Synthesize [label="Revise"]
}
```

```text #synthesize-prompt
You have received three independent code reviews of the same code. Retrieve the output of each review node using the available tools.

Produce a single unified report with these sections:

1. **Summary** — a brief overview of the code under review and the overall assessment.

2. **Prioritized Findings** — a numbered list of findings ordered by severity (critical → high → medium → low → informational). For each finding:
   - **Title**: concise name
   - **Severity**: critical / high / medium / low / informational
   - **Agreement**: note which reviewers flagged this (e.g., "All three", "A + B", "C only")
   - **Description**: what the issue is and why it matters
   - **Recommendation**: concrete fix or improvement

3. **Consensus & Disagreement** — highlight:
   - Areas where all three reviewers agreed
   - Areas where reviewers disagreed or only one reviewer flagged an issue, with brief reasoning about which perspective seems strongest

4. **Overall Recommendation** — a single clear recommendation (approve, approve with changes, or request revisions) based on the weight of evidence across all three reviews.

Before finalizing, validate each finding against the actual code. Pay particular attention to findings flagged by only one reviewer — these are the most likely to be either a genuine insight the others missed or a hallucinated issue. Read the relevant code to confirm the finding is real. Downgrade or drop any finding you cannot substantiate, and note when a single-reviewer finding was confirmed or rejected by your own inspection.

Be concise. Do not reproduce the full text of each review — synthesize and deduplicate.

Write the final report to `.stencila/reviews/` as a Markdown file. Derive the filename from the review target using kebab-case (e.g., `.stencila/reviews/codec-markdown-review.md`). Create the directory if it does not exist. If a file with that name already exists, chose another name.
```

```yaml #human-review
preamble: |
  The three parallel code reviews have been synthesized into a unified findings report.
  Please review the synthesis and decide whether to accept or request revisions.

questions:
  - header: Decision
    question: Is the synthesized review report acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
    finish-if: Accept

  - header: Revision Notes
    question: What should be changed in the synthesis?
    show-if: "human.decision == Revise"
    store: human.feedback
```

---

This page was generated from [`.stencila/workflows/code-review-parallel/WORKFLOW.md`](https://github.com/stencila/stencila/blob/main/.stencila/workflows/code-review-parallel/WORKFLOW.md).
