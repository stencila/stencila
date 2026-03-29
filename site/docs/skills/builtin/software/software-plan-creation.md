---
title: "Software Plan Creation Skill"
description: "Create delivery plans for software design specifications. Use when the user wants to plan implementation, break a design into delivery phases, create an implementation roadmap, produce a test plan, or structure a TDD approach for a feature. Reads the latest design spec by default and produces a phased plan covering implementation, testing, and documentation, with TDD slices sized to be logically coherent without degenerating into micro-slices."
keywords:
  - delivery plan
  - implementation plan
  - project plan
  - phased delivery
  - test plan
  - TDD
  - red green refactor
  - implementation roadmap
  - software planning
  - task breakdown
  - not code review
  - not design creation
  - not code generation
  - not implementation
---

Create delivery plans for software design specifications. Use when the user wants to plan implementation, break a design into delivery phases, create an implementation roadmap, produce a test plan, or structure a TDD approach for a feature. Reads the latest design spec by default and produces a phased plan covering implementation, testing, and documentation, with TDD slices sized to be logically coherent without degenerating into micro-slices.

**Keywords:** delivery plan · implementation plan · project plan · phased delivery · test plan · TDD · red green refactor · implementation roadmap · software planning · task breakdown · not code review · not design creation · not code generation · not implementation

> [!tip] Usage
>
> To use this skill, add `software-plan-creation` to the `allowed-skills` list in your agent's AGENT.md. You can also ask `#agent-creator` to build an agent that uses it.

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `ask_user` |

# Instructions

## Overview

Produce a delivery plan that turns a software design specification into an actionable sequence of implementation, testing, and documentation work. The primary input is a persisted design spec read from `.stencila/designs/`. The primary output is a Markdown plan written to `.stencila/plans/`.

Use this skill when the user wants a delivery plan, implementation plan, task breakdown, phased roadmap, or test-driven development strategy for a design spec. Do not use it when the main task is to write production code, create a design from scratch, review code, or build a workflow.

## Steps

1. Identify the design to plan against:
   - use `glob` with pattern `.stencila/designs/*.md` to discover available design specs (results are sorted newest first)
   - if the user names a specific design, locate it in the results
   - if the user does not name a design, use the first result (most recent)
   - use `read_file` to load the full design content
   - if no designs exist and the user is asking to plan an undesigned feature, handle the no-design case (see Edge Cases below)

2. Check for existing plans:
   - use `glob` with pattern `.stencila/plans/*.md` to see whether a plan already exists for the target design
   - if a matching plan exists, use `read_file` to load it and ask the user whether to replace it, update it, or create a new plan alongside it
   - do not silently overwrite an existing plan

3. Understand the design and codebase before planning:
   - read the design thoroughly before producing any plan content
   - identify the goals, scope, requirements, architecture, components, interfaces, acceptance criteria, and open questions
   - note any areas that are underspecified or flagged as open questions in the design — these become risks or prerequisites in the plan
   - if the design targets changes within an existing codebase, use `read_file`, `glob`, and `grep` to understand the relevant code structure, module boundaries, test conventions, and existing patterns — this context helps produce phases and tasks that align with how the codebase is actually organized

4. Determine phasing strategy:
   - assess whether the work is small enough for a single phase or benefits from incremental delivery
   - for multi-phase work, break delivery into phases where each phase produces a working, testable increment
   - order phases so that foundational work (data models, core interfaces, shared utilities) comes before dependent features
   - each phase should have a clear deliverable and exit criterion
   - avoid artificial phasing when the work is naturally a single coherent unit

5. Determine testing approach:
   - assess whether TDD (red-green-refactor) is appropriate for the work
   - TDD is a good fit when: the design has clear acceptance criteria, the work involves well-defined interfaces or data transformations, or correctness is critical
   - TDD is less appropriate when: the work is primarily exploratory, involves heavy UI prototyping, or depends on external systems that are hard to mock
   - when TDD fits, structure work as several logically coherent red-green-refactor slices rather than one broad slice or many trivial micro-slices
   - each slice should usually cover one behavior, one acceptance-criterion-sized increment, or one tightly related cluster of tests and implementation changes that naturally belong together
   - avoid micro-slicing where each slice captures only a tiny assertion, a purely mechanical follow-up, or a distinction so fine-grained that the workflow overhead would dominate the useful work
   - avoid over-broad slices that bundle several loosely related behaviors, multiple subsystem boundaries, or an amount of work that would make one Red-Green-Refactor cycle hard to review or recover when it fails
   - prefer a slice boundary when it corresponds to a meaningful behavior boundary, dependency boundary, risk boundary, or review checkpoint
   - it is acceptable for one slice to include a small number of closely related tests when they jointly define one coherent behavior and are likely to be implemented together
   - describe the red-green-refactor slices for each phase: what the first slice validates, what the next slice adds, and how the slices build incrementally while remaining substantial enough to justify a workflow iteration
   - when TDD does not fit, describe the testing strategy that does apply (integration tests, manual verification, end-to-end tests, etc.)

6. Size TDD slices deliberately:
   - use the codebase structure and design boundaries to decide slice size, not an abstract preference for ever-smaller units
   - prefer slices that a developer can usually complete in one focused iteration including Red, Green, Refactor, and review
   - group tightly related validation and implementation work in the same slice when separating them would create artificial overhead
   - keep separate slices when work crosses packages, layers, or risk boundaries, or when one behavior should be proven before another depends on it
   - if a phase's proposed slices read like a checklist of tiny assertions, merge them into fewer behavior-oriented slices
   - if a phase has only one huge slice covering many behaviors, split it into two or more coherent slices
   - for straightforward implementation work, a good default is a small number of meaningful slices per phase rather than a long tail of tiny ones
   - explicitly optimize for execution efficiency as well as conceptual clarity: the plan should support efficient workflow operation, not just theoretical decomposition

7. Draft the plan with these sections (adapt as needed):
   - **Title and summary**: what is being delivered and why
   - **Design reference**: which design spec this plan covers (name and path)
   - **Prerequisites**: anything that must be true before work begins (dependencies, access, open questions that must be resolved)
   - **Phases**: numbered phases, each with scope, deliverables, tasks, testing approach, and exit criteria
   - **Testing strategy**: overall approach including unit, integration, and end-to-end testing; TDD guidance where appropriate
   - **Documentation**: what documentation is needed and when it should be written (inline docs, API docs, user-facing docs, architecture decision records)
   - **Risks and mitigations**: derived from the design's open questions, assumptions, and complexity areas
   - **Definition of done**: what must be true for the entire plan to be considered complete

8. Tailor detail level to the design:
   - for a large design with many components, produce a detailed multi-phase plan
   - for a small, focused design, produce a concise single-phase plan
   - do not invent unnecessary phases or tasks to fill space
   - keep tasks concrete and actionable — each task should be something a developer can start and finish

9. Check plan quality before finishing:
   - every acceptance criterion from the design should be covered by at least one phase
   - phases should be ordered so no phase depends on work from a later phase
   - testing strategy should be realistic for the technology and architecture described
   - TDD slices should be logically coherent and execution-efficient: neither one giant batch nor a long series of trivial micro-slices
   - each TDD slice should map to a meaningful behavior or tightly related behavior cluster, not a single low-value assertion unless that assertion is itself the key risk boundary
   - documentation tasks should be included, not deferred indefinitely
   - risks from the design should be addressed or acknowledged

10. Persist the plan:
   - write the plan to `.stencila/plans/{name}.md` using `write_file`, where `{name}` is a kebab-case name derived from the design name (e.g., if the design is `user-auth-flow`, the plan might be `user-auth-flow-delivery`)
   - for updates to an existing plan, prefer `edit_file` or `apply_patch` over rewriting the entire file
   - provide the full Markdown plan content

## Suggested Output Structure

Use a structure like this when appropriate. Omit sections that do not apply and add sections that improve the plan.

```markdown
# Delivery Plan: <feature or system name>

## Summary

Brief description of what is being delivered and the value it provides.

## Design Reference

- **Design**: <design name>
- **Path**: <path to design file>

## Prerequisites

- <anything that must be resolved before implementation begins>

## Phases

### Phase 1: <name>

**Scope**: What this phase delivers.

**Tasks**:
1. <concrete task>
2. <concrete task>

**Testing**:
- <testing approach for this phase>
   - <TDD slices if applicable: slice 1 — write failing test(s) for one coherent behavior, implement, refactor; slice 2 — add the next meaningful behavior, implement, refactor; ...>

**Exit criteria**:
- <what must be true when this phase is complete>

### Phase 2: <name>

...

## Testing Strategy

### Unit tests
- ...

### Integration tests
- ...

### End-to-end tests
- ...

### TDD approach (when applicable)
- Work in red-green-refactor slices sized around meaningful behaviors
- Red: write one or a few closely related failing tests derived from one behavior or acceptance-criterion-sized increment
- Green: implement just enough code to pass those tests
- Refactor: clean up duplication and improve structure while tests stay green
- Repeat: move to the next slice — do not batch many unrelated tests before implementing, but also do not decompose work into low-value micro-slices
- Prefer slice boundaries that align with behavior, dependency, or risk boundaries
- This iterative cadence keeps each cycle focused while avoiding workflow overhead from over-fragmented plans

## Documentation

- <what documentation is needed and when>

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| ... | ... | ... |

## Definition of Done

- [ ] All acceptance criteria from the design are satisfied
- [ ] Tests pass at unit, integration, and end-to-end levels as appropriate
- [ ] Documentation is written and reviewed
- [ ] Code is reviewed and merged
```

## Examples

Input: "Create a delivery plan for the latest design"

Output:
- agent uses `glob` to find designs in `.stencila/designs/`, then `read_file` for the latest design
- produces a phased plan covering the design's components
- includes TDD guidance where the design has clear interfaces and acceptance criteria
- persists the plan to `.stencila/plans/` via `write_file`

Input: "Plan implementation for the user-auth-flow design using TDD"

Output:
- agent reads `.stencila/designs/user-auth-flow.md` via `read_file`
- produces a plan with explicit red-green-refactor slices per phase, sized around meaningful behaviors rather than micro-assertions
- persists the plan to `.stencila/plans/` via `write_file`

Concrete excerpt from a Phase 1 with balanced TDD slices:

```markdown
### Phase 1: Core authentication types and token validation

**Scope**: Define the `AuthToken` and `AuthError` types, implement token parsing and expiry validation.

**Tasks**:
1. Create `auth/types.rs` with `AuthToken` struct (fields: `sub`, `exp`, `iat`, `roles`) and `AuthError` enum
2. Implement `AuthToken::parse(raw: &str) -> Result<AuthToken, AuthError>`
3. Implement `AuthToken::is_expired(&self) -> bool`

**Testing (TDD slices)**:
- *Slice 1*: Write failing tests for malformed-token handling, including empty input and clearly invalid token structure. Implement just enough parsing and error handling to reject malformed input. Refactor.
- *Slice 2*: Write failing tests for successful parsing of a well-formed JWT into `AuthToken`, including the required `sub`, `exp`, `iat`, and `roles` fields. Extend parsing to pass. Refactor.
- *Slice 3*: Write failing tests for expiry behavior covering past and future `exp` values. Implement `is_expired` and any supporting time-handling logic. Refactor.

**Exit criteria**:
- `AuthToken` and `AuthError` types compile and are public
- All three TDD slices pass
- `cargo clippy` and `cargo fmt` produce no warnings
```

Input: "I need a plan for building a notification system" (no design exists)

Output:
- agent uses `glob` to search `.stencila/designs/` and finds no matching design
- advises the user to create a design first, explaining that a plan without a design spec will lack grounding
- optionally produces a lightweight preliminary plan with explicit caveats about missing design decisions, scope uncertainty, and assumptions

## Edge Cases

- **No designs exist**: Do not refuse. Explain that plans are most effective when based on a design spec. Offer two paths: (1) create a design first using the `software-design-creation` skill, or (2) proceed with a lightweight plan that explicitly marks all assumptions and flags that a design review should follow.
- **Design has open questions**: Include open questions as prerequisites or risks in the plan. Do not silently assume answers — flag them as decisions that must be resolved before the relevant phase begins.
- **Very small design**: Produce a single-phase plan. Do not artificially break trivial work into multiple phases.
- **Very large design**: Break into phases that each deliver a working increment. Prefer vertical slices (end-to-end functionality) over horizontal slices (all models, then all APIs, then all UI).
- **Plan naturally suggests many tiny TDD slices**: Merge adjacent low-risk, tightly related slices into fewer behavior-oriented slices so the plan remains efficient to execute.
- **Plan naturally suggests a very broad TDD slice**: Split it at behavior, dependency, subsystem, or risk boundaries so each slice remains reviewable and tractable.
- **Design lacks acceptance criteria**: Note this gap as a risk. Derive testable criteria from the requirements and goals sections where possible, and recommend the design be updated.
- **User asks for TDD but the work is unsuitable**: Explain why TDD may not be the best fit and propose an alternative testing strategy. Do not force TDD onto exploratory or heavily UI-driven work.
- **Plan already exists for this design**: If `glob` reveals an existing plan in `.stencila/plans/` that covers the target design, do not silently overwrite it. Show the user the existing plan name and ask whether to replace it entirely, update specific sections, or create a new plan alongside it with a different name.
- **Multiple designs match**: If the user's request is ambiguous and multiple designs could apply, list the candidates and ask which one to plan against rather than guessing.

## Slice Sizing Guidance

Use these heuristics when deciding whether a proposed TDD slice is too small, about right, or too large.

### Signs a slice is too small

- it captures only a single trivial assertion that adds little independent value
- it exists mainly because of implementation sequencing rather than a meaningful behavior boundary
- it would likely take less effort to execute than the workflow overhead around it
- several adjacent slices touch the same code, same acceptance criterion, and same failure mode

### Signs a slice is about right

- it delivers one meaningful behavior or one tightly related behavior cluster
- it has a clear failing-test story, a plausible minimal implementation, and a sensible refactor step
- it can be reviewed as one coherent increment
- if it fails, the corrective loop is still localized and understandable

### Signs a slice is too large

- it spans multiple loosely related behaviors or acceptance criteria
- it crosses subsystem or package boundaries without a strong reason
- it would require many unrelated tests before implementation can begin
- a failed implementation or review would generate diffuse feedback across too many concerns

When in doubt, choose the smallest slice that still feels like a meaningful increment of user-visible or developer-visible progress.

---

This page was generated from [`.stencila/skills/software-plan-creation/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/software-plan-creation/SKILL.md).
