---
name: software-plan-creation
description: Create delivery plans for software design specifications. Use when the user wants to plan implementation, break a design into delivery phases, create an implementation roadmap, produce a test plan, or structure a TDD approach for a feature. Reads the latest design spec by default and produces a phased plan covering implementation, testing, and documentation.
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
allowed-tools: read_file write_file edit_file apply_patch glob grep ask_user
---

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
   - when TDD fits, structure work as several narrow red-green-refactor slices rather than one broad slice — each slice should add one or a small number of tests, implement just enough code to pass them, and refactor before moving to the next slice; this keeps each cycle focused, maintains test quality, and avoids the pitfall of writing many tests at once that overwhelm the implementation step and dilute test intent
   - describe the red-green-refactor slices for each phase: what the first slice tests, what the next slice adds, and how the slices build on each other incrementally
   - when TDD does not fit, describe the testing strategy that does apply (integration tests, manual verification, end-to-end tests, etc.)

6. Draft the plan with these sections (adapt as needed):
   - **Title and summary**: what is being delivered and why
   - **Design reference**: which design spec this plan covers (name and path)
   - **Prerequisites**: anything that must be true before work begins (dependencies, access, open questions that must be resolved)
   - **Phases**: numbered phases, each with scope, deliverables, tasks, testing approach, and exit criteria
   - **Testing strategy**: overall approach including unit, integration, and end-to-end testing; TDD guidance where appropriate
   - **Documentation**: what documentation is needed and when it should be written (inline docs, API docs, user-facing docs, architecture decision records)
   - **Risks and mitigations**: derived from the design's open questions, assumptions, and complexity areas
   - **Definition of done**: what must be true for the entire plan to be considered complete

7. Tailor detail level to the design:
   - for a large design with many components, produce a detailed multi-phase plan
   - for a small, focused design, produce a concise single-phase plan
   - do not invent unnecessary phases or tasks to fill space
   - keep tasks concrete and actionable — each task should be something a developer can start and finish

8. Check plan quality before finishing:
   - every acceptance criterion from the design should be covered by at least one phase
   - phases should be ordered so no phase depends on work from a later phase
   - testing strategy should be realistic for the technology and architecture described
   - documentation tasks should be included, not deferred indefinitely
   - risks from the design should be addressed or acknowledged

9. Persist the plan:
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
- <TDD slices if applicable: slice 1 — write failing test for A, implement, refactor; slice 2 — write failing test for B, implement, refactor; ...>

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
- Work in narrow red-green-refactor slices, each covering one or a small number of tests
- Red: write one (or a few closely related) failing tests derived from an acceptance criterion
- Green: implement just enough code to pass those tests
- Refactor: clean up duplication and improve structure while tests stay green
- Repeat: move to the next slice — do not batch many tests before implementing
- This iterative cadence keeps each cycle small, maintains focus on test quality, and ensures tests are well-understood before the next slice begins

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
- produces a plan with explicit red-green-refactor slices per phase
- persists the plan to `.stencila/plans/` via `write_file`

Concrete excerpt from a Phase 1 with TDD slices:

```markdown
### Phase 1: Core authentication types and token validation

**Scope**: Define the `AuthToken` and `AuthError` types, implement token parsing and expiry validation.

**Tasks**:
1. Create `auth/types.rs` with `AuthToken` struct (fields: `sub`, `exp`, `iat`, `roles`) and `AuthError` enum
2. Implement `AuthToken::parse(raw: &str) -> Result<AuthToken, AuthError>`
3. Implement `AuthToken::is_expired(&self) -> bool`

**Testing (TDD slices)**:
- *Slice 1*: Write a failing test that `parse` returns `AuthError::MalformedToken` for an empty string. Implement just enough parsing to pass. Refactor.
- *Slice 2*: Write a failing test that `parse` returns a valid `AuthToken` for a well-formed JWT string. Extend parsing to pass. Refactor.
- *Slice 3*: Write a failing test that `is_expired` returns `true` when `exp` is in the past. Implement expiry check. Refactor.
- *Slice 4*: Write a failing test that `is_expired` returns `false` when `exp` is in the future. Verify existing implementation passes (no new code expected). Refactor if needed.

**Exit criteria**:
- `AuthToken` and `AuthError` types compile and are public
- All four TDD slices pass
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
- **Design lacks acceptance criteria**: Note this gap as a risk. Derive testable criteria from the requirements and goals sections where possible, and recommend the design be updated.
- **User asks for TDD but the work is unsuitable**: Explain why TDD may not be the best fit and propose an alternative testing strategy. Do not force TDD onto exploratory or heavily UI-driven work.
- **Plan already exists for this design**: If `glob` reveals an existing plan in `.stencila/plans/` that covers the target design, do not silently overwrite it. Show the user the existing plan name and ask whether to replace it entirely, update specific sections, or create a new plan alongside it with a different name.
- **Multiple designs match**: If the user's request is ambiguous and multiple designs could apply, list the candidates and ask which one to plan against rather than guessing.
