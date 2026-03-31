---
name: site-config-iterative
description: Create and iteratively refine a Stencila site configuration (`[site]` section in `stencila.toml`) using the `site-config-creator` and `site-config-reviewer` agents, route approved drafts through human review with optional commit, and loop on requested revisions
goal-hint: What site configuration do you want — what title, layout, features, social links, domain, or other settings should the site have?
keywords:
  - site config
  - stencila.toml
  - site settings
  - layout
  - navigation
  - search
  - domain
  - iterative
  - human-in-the-loop
when-to-use:
  - when a new site needs its `[site]` section configured in `stencila.toml`
  - when an existing site configuration needs to be updated or extended
  - when you want agent-based site config authoring followed by explicit human approval cycles
  - when configuring site layout, navigation, search, reviews, uploads, remotes, social links, or other site features
when-not-to-use:
  - when you only need a one-pass config change without review loops
  - when the task is to review an existing site configuration without modifying it
  - when modifying a theme (use `theme-creation-iterative` instead)
  - when the change is to workspace-level config outside `[site]` (e.g., `[workspace]`, `[models]`, `[mcp]`)
---

This workflow first uses the `site-config-creator` agent to draft or revise the `[site]` section of `stencila.toml`, then passes the draft to the `site-config-reviewer` agent for review. The reviewer uses the `workflow_set_route` tool to choose between the `Accept` and `Revise` edge labels; when it chooses `Revise` its response text contains concrete revision feedback.

The `Create` node uses the `workflow_get_output` tool to retrieve reviewer feedback and `workflow_get_context` with key `human.feedback` to retrieve human revision notes, so both automated and human guidance are available on iterative passes without bloating the prompt. After the reviewer accepts, the workflow enters a structured human review interview. Choosing `Revise` continues the interview to collect feedback (stored as `human.feedback` for the next creator pass). Choosing `Accept and Commit` routes through a `Commit` agent node that stages and commits the config changes before ending the workflow.

The `Create` node uses `persist="full"` so the creator agent's LLM session is reused across revision loops, avoiding the cost of re-exploring the workspace and re-reading files on every iteration. The `Review` node intentionally does not persist its session — a fresh session on each pass gives the reviewer unbiased "fresh eyes" on the current draft, avoiding anchoring on prior assessments that could mask regressions. The artifact being reviewed is a single file, so the re-read cost is low. A graph-wide `max-session-turns` default of 10 caps context growth.

```dot
digraph site_config_iterative {
  node [max-session-turns="10"]

  Start -> Create

  Create [agent="site-config-creator", prompt-ref="#creator-prompt", persist="full"]
  Create -> Review

  Review [agent="site-config-reviewer", prompt-ref="#reviewer-prompt"]
  Review -> HumanReview  [label="Accept"]
  Review -> Create       [label="Revise"]

  HumanReview [interview-ref="#human-review-interview"]
  HumanReview -> Commit  [label="Accept and Commit"]
  HumanReview -> End     [label="Accept"]
  HumanReview -> Create  [label="Revise"]

  Commit [agent="general", prompt-ref="#commit-prompt"]
  Commit -> End
}
```

```markdown #creator-prompt
Create or update the site configuration (`[site]` section of `stencila.toml`) for the goal:

$goal

Before starting, use `workflow_get_output` to check for reviewer feedback from a previous iteration. If feedback is present, use it to revise the existing draft instead of starting over. If you disagree with a specific finding, you may provide a reasoned rebuttal instead of incorporating it.

Also use `workflow_get_context` with key "human.feedback" to check for human revision notes and incorporate those as well.
```

```markdown #reviewer-prompt
Review the current site configuration (`[site]` section of `stencila.toml`) for the goal:

$goal

If the configuration is acceptable, choose the `Accept` branch. If the configuration needs changes, choose the `Revise` branch and provide specific revision feedback in your response.
```

```yaml #human-review-interview
preamble: |
  The site-config-reviewer agent has approved the current site configuration.
  Please review the configuration and decide whether to accept it or send it back for revision.

questions:
  - header: Decision
    question: Is the site configuration acceptable?
    type: single-select
    options:
      - label: Accept and Commit
      - label: Accept
      - label: Revise
    store: human.decision

  - header: Revision Notes
    question: What specific changes or improvements should be made?
    store: human.feedback
    show-if: "human.decision == Revise"
```

```markdown #commit-prompt
Commit the site configuration changes.

Site config goal: $goal

**Step 1: stage changes**

Use the shell tool to review uncommitted changes with `git status` and `git diff --stat`.
Stage the `stencila.toml` file. Use the goal description as a guide, but include any other
files that are clearly part of this site configuration work. Avoid staging unrelated changes.

**Step 2: commit**

Compose a commit message based on the site config goal and the actual changes staged.
Inspect the repository's recent commit history (`git log --oneline -20`) to infer the
project's commit message conventions and follow them. Also check for any commit message
instructions in the system prompt or prior context and apply those.
Run `git commit` with the composed message.

If any step fails (nothing to commit, git errors, etc.), report the issue but do not block
the workflow — execution will continue regardless.
```
