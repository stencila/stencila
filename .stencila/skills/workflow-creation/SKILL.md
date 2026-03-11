---
name: workflow-creation
description: Create a new Stencila workflow. Use when asked to create, write, scaffold, or set up a workflow directory or WORKFLOW.md file. Covers workflow discovery, duplicate-name checks, ephemeral workflows, WORKFLOW.md frontmatter, DOT pipeline authoring, goals, agents, branching, and validation.
keywords:
  - workflow
  - pipeline
  - create
  - scaffold
  - write
  - WORKFLOW.md
allowed-tools: read_file write_file edit_file apply_patch glob grep shell ask_user list_agents
---

## Overview

Create a new workflow directory and `WORKFLOW.md` file for Stencila. A workflow is a directory under `.stencila/workflows/` containing a `WORKFLOW.md` file with YAML frontmatter and a Markdown body. The body usually begins with a `dot` fenced code block that defines the pipeline, followed by optional human-readable documentation.

Use this skill when the user wants to define a multi-stage AI workflow, orchestrate several agent or human steps, or scaffold a reusable pipeline that can be validated and run with Stencila.

## Steps

1. Determine the workflow name, description, and intended goal from the user's request
2. Validate the name against the naming rules below
3. Resolve the closest workspace by walking up from the current directory to find the nearest `.stencila/` directory; if none exists, use the repository root or current working directory and create `.stencila/workflows/<name>/`
4. Ask clarifying questions if the workflow's stages, branching behavior, agents, or goal are unclear
5. Check whether a workflow with the same name already exists in the target workspace; if it does, ask whether to overwrite, merge, or abort before changing anything
6. Decide whether the workflow should be permanent or ephemeral:
   - default to a normal permanent workflow unless the user asks for a temporary workflow or the creating tool explicitly uses ephemeral creation
   - prefer ephemeral workflows for agent-created drafts, quick experiments, or workflows the user may want to discard after immediate use
   - if ephemeral, plan to mark the workflow directory using a `.gitignore` sentinel file containing `*`
7. Create the directory `<closest-workspace>/.stencila/workflows/<name>/`
8. Write `WORKFLOW.md` with:
   - YAML frontmatter containing at least `name` and `description`
   - `goal` in frontmatter when the user provides a stable high-level objective; duplicate it in graph attributes only when required by execution semantics or existing project conventions
   - `keywords` with domain-relevant terms and `when-to-use`/`when-not-to-use` entries to improve discoverability and delegation accuracy
   - A Markdown body whose first `dot` fenced code block contains the workflow pipeline
   - Optional surrounding Markdown documentation that explains the workflow to humans
9. If ephemeral, create the `.gitignore` sentinel file with exactly `*` on its own line; if permanent, do not add that sentinel
10. Prefer a simple linear pipeline first, then add branching, retry loops, conditions, human review, or agent overrides only when the user asks for them or the workflow clearly needs them
11. Use `list_agents` when agent selection matters so you can choose from available specialized agents instead of guessing names
12. When using `list_agents`, prefer agents whose metadata supports the node's role: use `description` to check the agent's core capability, `keywords` to match domain terms and user intent, `when-to-use` for positive fit signals, and `when-not-to-use` to avoid poor matches
13. Reference existing agents by name with the `agent` attribute when appropriate, preferring specialized agents returned by `list_agents` when their metadata indicates a good fit; do not invent agent names unless the user requests them or they are already clear project conventions
14. Replace placeholders such as `TODO` before considering the workflow complete
15. Validate the finished workflow with `stencila workflows validate <name>`, the workflow directory path, or the `WORKFLOW.md` path, and report the result to the user when possible

When working from a nested directory in a repository, create the workflow in the closest workspace's `.stencila/workflows/` directory rather than creating a new `.stencila/` tree under the current subdirectory.

## Naming Rules

Workflow names must be **lowercase kebab-case**:

- 1–64 characters
- Only lowercase alphanumeric characters and hyphens (`a-z`, `0-9`, `-`)
- Must not start or end with a hyphen
- Must not contain consecutive hyphens (`--`)
- Must match the parent directory name
- Pattern: `^[a-z0-9]([a-z0-9-]{0,62}[a-z0-9])?$`

By convention, workflow names should describe the **end-to-end process** the workflow accomplishes, not the exact sequence of steps in the graph.

Use these naming patterns:

- `thing-process` for the default case
- `thing-process-approach` when you need to distinguish multiple workflows for the same process

Where:

- `thing` is the artifact or domain the workflow acts on, such as `code`, `blog`, `agent`, or `schema`
- `process` is the broad lifecycle stage or end-to-end goal, such as `generation`, `refinement`, `publication`, `review`, or `creation`
- `approach` is an optional qualifier for strategy or tradeoffs, such as `quick`, `iterative`, `consensus`, `thorough`, or `guided`

Prefer names that communicate purpose rather than pipeline shape. Avoid brittle names that list every step, such as `create-review-refine-test-deploy` or `plan-implement-validate`, because they become outdated as the workflow evolves.

Good examples:

- `code-review`
- `code-generation-iterative`
- `blog-generation-quick`
- `architecture-design-consensus`
- `agent-creation-guided`

Common corrections: `workflowBuilder` → `workflow-builder`, `test_deploy` → `test-deploy`, `Code-Review` → `code-review`.

## WORKFLOW.md Format

The file has two parts:

1. **YAML frontmatter** between `---` delimiters — metadata such as `name`, `description`, and optional `goal`
2. **Markdown body** — a DOT pipeline in the first `dot` fenced code block, plus optional documentation

### Required frontmatter fields

- `name` — the workflow name (must match directory name)
- `description` — what the workflow does and when to use it; keep it concise and specific

### Optional frontmatter fields

- `goal` — a high-level objective for the workflow; prefer this location for stable intent that prompts interpolate as `$goal`
- `keywords` — list of keywords or tags for discovery and routing; use terms that reflect the workflow's domain and purpose
- `when-to-use` — list of positive selection signals describing when this workflow should be used; helps managers choose the right workflow
- `when-not-to-use` — list of negative selection signals describing when this workflow should not be used
- `license` — SPDX identifier or reference to a license file if needed
- `compatibility` — environment requirements (max 500 characters)
- `metadata` — arbitrary key-value pairs if the workflow needs extra structured metadata

Ephemeral status is not stored in frontmatter. It is determined by whether the workflow directory has a `.gitignore` sentinel file containing exactly `*`.

### DOT Pipeline Expectations

- Put the executable pipeline in the first `dot` fenced code block in the Markdown body
- Use a directed graph such as `digraph code_review { ... }`
- Prefer frontmatter `goal` for the workflow objective when it is known and stable
- Add a graph attribute like `graph [goal="..."]` only when required by execution semantics or to match an existing project style
- Use node attributes such as `prompt`, `agent`, and `ask` where needed
- When prompts, shell scripts, or human questions become long or multiline, prefer reusable fenced code blocks with ids and reference them from the graph using kebab-case attributes `prompt-ref`, `shell-ref`, `ask-ref`, and `interview-ref` instead of embedding long escaped strings directly in DOT
- Do not use refs for short single-line values just for consistency; use them when they materially improve readability and maintainability
- When a human review step needs to collect multiple pieces of information (e.g., a decision and freeform feedback), use `interview-ref` pointing to a YAML code block that defines the full interview with preamble, multiple typed questions, and per-question `store` keys; continue to use `ask` or `ask-ref` for single-question human gates
- Use edges to express sequencing, branching, retry loops, and approval paths
- Prefer the house style of placing the entry edge near the top, then organizing each node as a block: node definition followed immediately by its outgoing edge or edges
- Keep the initial scaffold minimal and readable unless the user explicitly asks for a complex pipeline

Markdown content outside the first DOT block is documentation for humans. Only the first DOT block is extracted as the pipeline definition.

Reusable content references resolve against code blocks or code chunks with ids in the same `WORKFLOW.md`. Use them mainly for longer content, for example:

````markdown
```dot
digraph example {
    Create [agent="writer", prompt-ref="#creator-prompt"]
    Check  [shell-ref="#run-checks"]
    Ask    [ask-ref="#human-question", question-type="freeform"]
    Review [interview-ref="#review-interview"]
}
```

```text #creator-prompt
Create or revise the draft for this goal: $goal

...

Incorporate reviewer feedback when present:
$last_output
```

```sh #run-checks
cargo fmt -p workflows
...
cargo test -p workflows
```

```text #human-question
What should change before the next revision?

Be specific about missing sections, unclear instructions, or structural issues.
```

```yaml #review-interview
preamble: |
  Please review the draft and provide structured feedback.

questions:
  - question: "Is the draft ready to publish?"
    header: Decision
    question_type: multiple_choice
    options:
      - label: Approve
      - label: Revise
    store: review.decision

  - question: "What specific changes should be made?"
    header: Feedback
    question_type: freeform
    store: review.feedback
```
````

Ids must be unique within the document.

## Ephemeral Workflows

An ephemeral workflow is a temporary workflow directory under `.stencila/workflows/` that includes a `.gitignore` file containing exactly:

```text
*
```

This sentinel marks the workflow as temporary without adding any special frontmatter or DOT attributes.

Use ephemeral workflows when:

- the workflow is being created by an agent for immediate execution
- the user wants a draft or throwaway workflow
- the workflow should be easy to discard if the user does not keep it

Do not make a workflow ephemeral unless the user asks for temporary behavior or the surrounding flow clearly implies it.

When describing the result to the user:

- explain whether the workflow is ephemeral or permanent
- if ephemeral, mention that it can be kept with `stencila workflows save <name>`
- if ephemeral, mention that it can be removed with `stencila workflows discard <name>`

## Common Workflow Patterns

House style for examples in this skill:

- use frontmatter with `name`, `description`, and `goal` when the objective is stable
- omit extra Markdown headings unless they add important human-facing documentation
- use `Start` and `End` nodes for readability
- place the `Start -> ...` entry edge near the top, then for each node place the node definition before its outgoing edge or edges
- use simple edge labels such as `Pass`, `Fail`, `Approve`, and `Revise`
- use `$goal` in prompts when the workflow has a frontmatter `goal`

### Linear workflow

````markdown
---
name: lit-review
description: Search and summarize recent literature
goal: Review recent literature on CRISPR gene editing
---

```dot
digraph lit_review {
    Start -> Search
    
    Search    [prompt="Search for recent papers on: $goal"]
    Search -> Summarize

    Summarize [prompt="Summarize the key findings across the papers"]
    Summarize -> Draft

    Draft     [prompt="Draft a literature review from the summaries"]
    Draft -> End
}
```
````

### Agent-driven workflow with review gate

````markdown
---
name: code-review
description: Automated code review with human approval gate
goal: Implement and review the feature
---

```dot
digraph code_review {
    Start -> Design

    Design [agent="code-planner", prompt="Design the solution for: $goal"]
    Design -> Build

    Build  [agent="code-engineer", prompt="Implement the design"]
    Build -> Test

    Test   [agent="code-tester", prompt="Run tests and validate"]
    Test -> Review       [label="Pass", condition="outcome=success"]
    Test -> Build        [label="Fail", condition="outcome!=success"]

    Review [shape=human]
    Review -> End        [label="Approve"]
    Review -> Design     [label="Revise"]
}
```
````

### Agent-driven workflow with structured review interview

````markdown
---
name: code-review-guided
description: Automated code review with structured human feedback
goal: Implement and review the feature with detailed feedback
---

```dot
digraph code_review_guided {
    Start -> Design

    Design [agent="code-planner", prompt="Design the solution for: $goal"]
    Design -> Build

    Build  [agent="code-engineer", prompt="Implement the design"]
    Build -> Review

    Review [interview-ref="#review-interview"]
    Review -> End        [label="Approve"]
    Review -> Design     [label="Revise"]
}
```

```yaml #review-interview
preamble: |
  Please review the implementation and provide structured feedback.

questions:
  - question: "Is the implementation ready to merge?"
    header: Decision
    question_type: multiple_choice
    options:
      - label: Approve
      - label: Revise
    store: review.decision

  - question: "What specific changes should be made?"
    header: Feedback
    question_type: freeform
    store: review.feedback
```
````

## Authoring Guidance

- Start from the user's real objective, then map it to stages such as research, plan, build, test, review, and publish
- Use `Start` and `End` nodes for readability unless the workflow format or user request suggests a different style
- Use `list_agents` before assigning non-obvious agents so the workflow can reference real available agents rather than guessed names
- When reviewing `list_agents` results, choose agents by metadata rather than by name alone: match `description` to the node's responsibility, use `keywords` for domain and task terms, treat `when-to-use` as positive routing guidance, and use `when-not-to-use` to avoid agents that are a poor fit
- Use `agent="name"` to reference existing agents by name; when available, prefer specialized agents whose metadata indicates they match the node's task. Stencila resolves workspace agents first, then user-level agents, then CLI-detected agents
- When a node has no `agent` attribute, the engine uses a default agent; this fallback is unlikely to be optimal for a well-designed workflow, so prefer explicit agent selection unless the user wants a minimal draft
- Use inline `agent.*` dotted attributes only when the user explicitly wants node-specific overrides such as `agent.model`, `agent.provider`, or `agent.reasoning-effort`
- Use `shape=human` for explicit human approval or review steps
- Put reusable high-level intent in frontmatter `goal` and refer to it in prompts with `$goal`
- Prefer frontmatter `goal` over repeating the same objective in both frontmatter and graph attributes
- Prefer explicit edge labels and conditions when a branch depends on success, failure, approval, or revision
- Keep each node's outgoing routing close to that node in the DOT source instead of separating all edges from all node definitions
- Do not try to encode ephemeral status in frontmatter or the DOT graph; use the `.gitignore` sentinel instead when needed
- Do not overcomplicate the first draft; a shorter valid workflow is better than an elaborate but unclear one
- Do not encode every node or branch in the workflow name; keep naming focused on the process and, if needed, a broad approach modifier
- Use `interview-ref` when a human review step needs to collect both a routing decision and structured feedback in a single pause; ensure every freeform question has a `store` key so answers are not silently lost
- Routing in multi-question interviews is driven by the first `multiple_choice` question's answer, matched against outgoing edge labels — keep routing edges visible in the DOT graph, not hidden in the YAML spec

## Practical Workflow Design Guidance

Design the workflow so that each stage makes visible progress toward the goal instead of just adding more prompts.

- Break broad objectives into stages that reduce uncertainty or produce a concrete artifact for the next step
- For each non-trivial node, be able to state its input, output, success condition, and revision path
- Prefer node prompts that describe the local task; use frontmatter `goal` for the stable overall objective
- After major generative steps, add a test, review, critique, or approval gate when the next action depends on quality
- Add loops only when a later node can provide specific feedback that improves an earlier node
- Base branches on meaningful decisions such as pass/fail, approve/revise, or sufficient/insufficient evidence
- Use human approval when the workflow crosses a trust boundary such as publish, deploy, or accept consequential changes
- If a stage does not change what the workflow knows, decides, or produces, it is usually unnecessary

## Workflow Design Heuristics by Objective Type

Use patterns like these as a starting point, then simplify or extend them to fit the request:

| Objective type | Convergence-oriented shape |
|---|---|
| Research / literature review | clarify question → search → extract evidence → synthesize → critique gaps → draft |
| Coding / implementation | clarify requirements → design → implement → test → review → revise or approve |
| Publishing / editorial | brief → draft → edit → fact-check → approve → publish |
| Decision support | define criteria → gather options → evaluate → compare → recommend → approve |
| Data analysis | define question → collect data → clean/validate → analyze → interpret → review |

In each pattern, try to alternate generation and evaluation so later steps decide whether earlier work is good enough to continue.

## Example Walkthrough

Input: "Create a workflow that designs, implements, tests, and then asks for human approval before finishing"

Process:

1. Derive name: prefer a process-oriented name such as `code-generation-iterative` rather than a step-by-step name such as `plan-implement-validate`
2. Resolve workspace: find the nearest `.stencila/` directory, for example at the repository root
3. Target path: `.stencila/workflows/code-generation-iterative/WORKFLOW.md`
4. Check whether `.stencila/workflows/code-generation-iterative/` already exists; if it does, ask whether to overwrite, merge, or abort
5. Capture the goal and choose a pipeline with design, build, test, and human review steps
6. Use a DOT graph with clear edges, simple branch labels, and prompts or agents for each non-human node
7. Write the file, then validate it

Output:

````markdown
---
name: code-generation-iterative
description: Generate and refine a requested software change through design, implementation, testing, and review
goal: Implement and validate the requested feature
---

```dot
digraph code_generation_iterative {
    Start -> Design

    Design [prompt="Design an implementation plan for: $goal"]
    Design -> Build

    Build  [prompt="Implement the approved design"]
    Build -> Test

    Test   [prompt="Run or describe validation steps and report the outcome"]
    Test -> Review   [label="Pass", condition="outcome=success"]
    Test -> Build    [label="Fail", condition="outcome!=success"]

    Review [shape=human]
    Review -> End    [label="Approve"]
    Review -> Design [label="Revise"]
}
```
````

Validated with: `stencila workflows validate plan-implement-validate`

## Ephemeral Example

Input: "Create a temporary workflow I can try once to summarize a set of notes"

Process:

1. Derive name: `note-summary`
2. Resolve the nearest workspace and target `.stencila/workflows/note-summary/`
3. Confirm this should be ephemeral rather than permanent
4. Check whether the target directory already exists; if it does, ask whether to overwrite, merge, or abort
5. Write `WORKFLOW.md` using the same frontmatter and DOT house style as the other examples
6. Create `.gitignore` containing exactly `*`
7. Validate the workflow and explain how to save or discard it

Output structure:

```text
.stencila/workflows/note-summary/
├── .gitignore   # contains exactly: *
└── WORKFLOW.md
```

Example `WORKFLOW.md`:

````markdown
---
name: note-summary
description: Summarize a temporary set of notes
goal: Summarize the provided notes into a concise brief
---

```dot
digraph note_summary {
    Start -> Summarize -> End

    Summarize [prompt="Summarize the notes for: $goal"]
}
```
````

Notes to include when reporting completion:

- this workflow is ephemeral because the directory contains the `.gitignore` sentinel
- keep it with `stencila workflows save note-summary`
- remove it with `stencila workflows discard note-summary`

Validated with: `stencila workflows validate note-summary`

## Edge Cases

- **Workflow directory already exists**: Ask the user whether to overwrite, merge, or abort before modifying an existing workflow. Never silently overwrite.
- **Name mismatch**: If the requested name is not valid kebab-case, suggest a corrected version rather than failing silently.
- **Nested workspaces**: If multiple `.stencila/` directories exist in the ancestor chain, use the nearest one. Do not create a duplicate `.stencila/workflows/` tree.
- **Empty or placeholder content**: Do not consider the workflow complete if any `TODO`, `<placeholder>`, or empty `description` remains in the final `WORKFLOW.md`.
- **No DOT block**: A workflow without a DOT block may still be partially drafted, but it is incomplete for execution; add a valid first `dot` block before reporting completion unless the user explicitly asks for documentation only.
- **Missing goal**: `goal` is optional. Omit it if the user has not provided a stable overarching objective.
- **Unknown agents**: If the workflow references agent names that may not exist, tell the user they need corresponding agents or remove the `agent` attributes in the initial scaffold.
- **Overriding agent properties**: Use inline `agent.*` attributes sparingly; prefer reusable agent definitions unless the user clearly needs a node-specific override.
- **Ephemeral status**: Do not add custom frontmatter like `ephemeral: true`; ephemeral workflows are identified solely by the `.gitignore` sentinel file containing `*`.

## Validation

Before finishing, validate the workflow:

```sh
# By workflow name
stencila workflows validate <workflow-name>

# By directory path
stencila workflows validate .stencila/workflows/<workflow-name>

# By WORKFLOW.md path
stencila workflows validate .stencila/workflows/<workflow-name>/WORKFLOW.md
```

Validation should pass before you report the workflow as complete.

## Limitations

- This skill covers workflow structure, metadata, and authoring conventions. It does not execute the workflow or verify the runtime behavior of referenced agents.
- Validation checks structure and known conventions, but some design issues may only become apparent during execution or real use.
