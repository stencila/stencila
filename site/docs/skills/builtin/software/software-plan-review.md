---
title: "Software Plan Review Skill"
description: "Critically review a software delivery plan and suggest concrete improvements. Use when the user wants to review, critique, audit, evaluate, or strengthen a delivery plan, implementation plan, project plan, phased roadmap, or test plan. Reviews plans produced by the software-plan-creation skill, checking task breakdown quality, sequencing, dependencies, testing strategy, TDD slice design and sizing, design coverage, risks, and actionability. Produces a structured critique with prioritized recommendations."
keywords:
  - plan review
  - plan critique
  - delivery plan
  - implementation plan
  - task breakdown
  - risk assessment
  - testing strategy
  - TDD slice review
  - definition of done
  - not code review
  - not design creation
---

Critically review a software delivery plan and suggest concrete improvements. Use when the user wants to review, critique, audit, evaluate, or strengthen a delivery plan, implementation plan, project plan, phased roadmap, or test plan. Reviews plans produced by the software-plan-creation skill, checking task breakdown quality, sequencing, dependencies, testing strategy, TDD slice design and sizing, design coverage, risks, and actionability. Produces a structured critique with prioritized recommendations.

**Keywords:** plan review · plan critique · delivery plan · implementation plan · task breakdown · risk assessment · testing strategy · TDD slice review · definition of done · not code review · not design creation

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `glob`, `grep` |

# Instructions

## Overview

Review an existing software delivery plan with the goal of making it clearer, more complete, more correct, and more actionable for implementation. This skill complements the `software-plan-creation` skill — it evaluates plans that skill produces. The primary output is a structured critique that identifies strengths, weaknesses, risks, sequencing problems, and concrete improvements.

Prefer critique plus revision guidance over rewriting the full plan unless the user explicitly asks for a rewritten version.

Use this skill when the user already has a plan artifact and wants critical review rather than initial drafting. Do not use it when the main task is to write production code, create a design from scratch, review source code, or create a new plan.

## Expected Plan Structure

Plans produced by the `software-plan-creation` skill follow this structure. Use it as the reference when checking whether sections are present and well-formed:

1. **Title and Summary** — what is being delivered and the value it provides
2. **Design Reference** — the source design name and path for traceability
3. **Prerequisites** — dependencies, access, open questions that must be resolved before work begins
4. **Phases** — numbered phases, each containing:
   - **Scope** — what the phase delivers
   - **Tasks** — numbered, concrete, actionable tasks
   - **Testing** — testing approach for the phase, including TDD slices when applicable
   - **Exit criteria** — what must be true when the phase is complete
5. **Testing Strategy** — overall approach covering unit, integration, and end-to-end testing; TDD guidance where applicable
6. **Documentation** — what documentation is needed and when
7. **Risks and Mitigations** — table with Risk, Impact, and Mitigation columns
8. **Definition of Done** — checklist of conditions for the entire plan to be considered complete

Not every plan will have all sections. Assess whether omitted sections are genuinely unnecessary or represent gaps.

## Steps

1. Identify the plan to review:
   - accept a plan name, a stored plan reference, or plan text pasted in the conversation
   - restate the plan's apparent purpose, the design it covers, and the scope of work
   - if the input is fragmented, reconstruct the main proposal before critiquing it

2. Resolve the plan artifact:
   - if the full plan is already present in the conversation, review that text directly — do not force retrieval through plan tools
   - if the user refers to a stored plan by name or asks to review an existing plan, use `glob` with pattern `.stencila/plans/*.md` to locate candidates
   - use `read_file` to load the selected stored plan
   - if multiple similarly named plans exist, list candidates, compare them, and review the one that best matches the user's request

3. Resolve the corresponding design (when available):
   - use `glob` with pattern `.stencila/designs/*.md` to find the design, then `read_file` to load the design specification that the plan is based on
   - if the plan includes a "Design Reference" section, use the name or path it provides to locate the design
   - reviewing a plan against its source design is essential for checking coverage, alignment, and whether acceptance criteria are fully addressed
   - if no corresponding design exists or cannot be identified, note this as a limitation and review the plan on its own merits

4. Understand the plan before judging it:
   - summarize the planned work, its phases, and the overall delivery strategy in plain language
   - identify the stated goals, prerequisites, phasing strategy, testing approach, and definition of done
   - note any missing context that materially limits confidence in the review

5. Evaluate structure and completeness against the expected plan format:
   - check whether each expected section is present and well-formed
   - check that the summary accurately reflects the design's goals
   - check that the design reference is present and identifies the source design clearly enough to trace back
   - check that prerequisites include open questions from the design
   - flag missing sections and assess whether their absence is justified or is a gap

6. Evaluate task breakdown and granularity:
   - assess whether tasks are concrete and actionable — each task should be something a developer can start and finish
   - flag tasks that are too vague (e.g., "implement the feature") or too granular (e.g., individual lines of code)
   - check whether each phase has a clear deliverable and exit criterion
   - assess whether the number of phases is appropriate — not artificially inflated for simple work or too few for complex work
   - check whether every acceptance criterion from the design maps to at least one task

7. Evaluate phasing and sequencing:
   - check whether phases are ordered so that foundational work precedes dependent work
   - identify circular dependencies or phases that depend on work from a later phase
   - assess whether vertical slices (end-to-end functionality) are preferred over horizontal slices (all models, then all APIs, then all UI) where appropriate
   - flag cases where parallelizable work is unnecessarily serialized
   - check that each phase produces a working, testable increment

8. Evaluate testing strategy:
   - assess whether the overall testing approach is realistic for the technology and architecture described
   - check whether testing is specified for each phase, not just at the plan level
   - check whether unit, integration, and end-to-end testing are covered as appropriate for the work
   - flag phases that have no testing approach specified

9. Evaluate TDD slices (when TDD is used):
   - check whether TDD slices follow logically coherent red-green-refactor cycles — each slice should cover one meaningful behavior or a tightly related behavior cluster, implement just enough to pass, then refactor
   - flag slices that batch too many unrelated tests before implementing (this dilutes test intent and overwhelms the implementation step)
   - flag slices that are overly micro-sliced into trivial assertions or mechanical follow-ups where workflow overhead would dominate the useful work
   - check whether slices are well-sequenced and build incrementally on each other
   - check whether slice descriptions are specific enough to act on (e.g., "write a failing test that `parse` returns `AuthError::MalformedToken` for an empty string" is good; "write tests for parsing" is too vague)
   - assess whether slice boundaries align with behavior boundaries, dependency boundaries, risk boundaries, or meaningful review checkpoints
   - suggest merging adjacent slices when they are too small and tightly related, or splitting slices when they are too broad and mix loosely related behaviors
   - if TDD is proposed for work where it is a poor fit (exploratory, UI-heavy, hard-to-mock external dependencies), flag this and suggest an alternative

10. Evaluate risks and mitigations:
    - check whether risks from the source design are acknowledged and addressed in the plan
    - identify risks the plan introduces that were not in the design (e.g., risky sequencing, single points of failure, missing expertise)
    - assess whether mitigations are concrete rather than generic
    - if the plan uses the expected table format (Risk | Impact | Mitigation), check that the Impact column is meaningful and mitigations are actionable
    - check whether the hardest parts of the plan are surfaced rather than hidden

11. Evaluate definition of done:
    - check whether the definition of done is present, specific, and verifiable
    - check whether it aligns with the design's acceptance criteria
    - flag checklist items that are vague (e.g., "code is good") or missing (e.g., no mention of tests passing, documentation, or code review)

12. Evaluate alignment with the source design:
    - check whether the plan's scope matches the design's scope — not broader, not narrower
    - verify that the plan does not silently drop design requirements or acceptance criteria
    - verify that the plan does not introduce scope that was explicitly out-of-scope in the design
    - check whether open questions from the design are handled as prerequisites or risks in the plan

13. Evaluate documentation tasks:
    - check whether documentation tasks are included in the plan
    - assess whether documentation is integrated into phases rather than deferred to the end
    - check whether the plan specifies what kind of documentation is needed (inline docs, API docs, user-facing docs, architecture decision records)

14. Produce a structured review report following the Report Format below

15. Distinguish facts from uncertainty:
    - clearly label assumptions made during the review
    - separate definite problems from possible risks or questions that need confirmation
    - avoid inventing system facts that are not supported by the plan or its source design

## Review Checklist

Assess the plan against the following dimensions, tailoring depth to the size and complexity of the plan.

### Summary and Design Reference

- Is there a clear summary of what is being delivered and why?
- Is the source design identified with name and path so the plan can be traced back?
- Does the summary accurately reflect the design's goals?

### Prerequisites

- Are prerequisites clearly stated?
- Are open questions from the design listed as prerequisites or risks?
- Are external dependencies, access requirements, or decisions that must be resolved before work begins identified?

### Task Breakdown and Granularity

- Are tasks concrete, actionable, and appropriately scoped?
- Could a developer pick up any task and understand what to do?
- Are there tasks that are too vague to estimate or begin?
- Are there tasks that are so granular they add noise rather than clarity?
- Does every acceptance criterion from the design map to at least one task?

### Phasing and Sequencing

- Does each phase produce a working, testable increment?
- Are phases ordered so no phase depends on work from a later phase?
- Is the number of phases appropriate for the complexity of the work?
- Are vertical slices preferred over horizontal slices where appropriate?
- Are there opportunities to parallelize work that the plan serializes?

### Testing Strategy

- Is the testing approach specified for each phase?
- Is the overall testing strategy realistic for the architecture?
- Are unit, integration, and end-to-end testing covered as appropriate?
- If TDD is used, are slices logically coherent and incremental?
- If TDD is used, does each slice specify what test to write, what to implement, and what to refactor?
- If TDD is used, do slices avoid batching many tests before implementing?
- If TDD is used, do slices also avoid low-value micro-slicing where several adjacent slices should really be one behavior-oriented unit?
- If TDD is used, are slice boundaries justified by behavior, dependency, subsystem, or risk boundaries?
- If TDD is not used, is the alternative clearly described?
- Are there phases with no testing approach specified?

### Documentation

- Are documentation tasks included in the plan?
- Is it clear what documentation is needed and when it should be written?
- Are documentation tasks integrated into phases rather than deferred to the end?

### Risks and Mitigations

- Are risks from the source design addressed or acknowledged?
- Does the plan identify risks it introduces (sequencing risks, dependency risks, etc.)?
- Are mitigations concrete and actionable rather than generic?
- Are the hardest parts of the plan surfaced rather than hidden?
- If using the table format, are Impact values meaningful?

### Definition of Done

- Is there a clear definition of done for the entire plan?
- Does the definition of done align with the design's acceptance criteria?
- Is the definition of done specific and verifiable?
- Does it cover tests passing, documentation, and code review?

### Feasibility and Actionability

- Is the plan realistic given constraints, dependencies, and delivery goals?
- Could a team use this plan to estimate, assign, and begin work?
- Are open questions clearly separated from settled decisions?
- Are next changes to the plan obvious from the review?

## Report Format

Structure the review as follows.

### Overall Assessment

One to three sentences summarizing the plan's current quality and the most important improvement needed.

### Strengths

A short bullet list of what the plan already does well. Recognizing strengths helps the author know what to preserve.

### Findings

Group findings under these headings when relevant (omit headings with no findings):

- **Structure and completeness** — missing or malformed sections
- **Design coverage** — dropped requirements, added scope, or misalignment with the source design
- **Task breakdown and granularity** — vague, oversized, or overly granular tasks
- **Phasing and sequencing** — ordering issues, dependency problems, artificial phasing
- **Testing strategy** — missing coverage, unrealistic approaches, TDD slice problems
- **Documentation** — missing or deferred documentation tasks
- **Risks and mitigations** — unaddressed risks, generic mitigations
- **Definition of done** — missing, vague, or misaligned completion criteria

For each finding:

- indicate severity as **High**, **Medium**, or **Low**
- describe the issue precisely, referencing the specific phase, task, or section
- explain why it matters

### Recommendations

Provide a numbered list of concrete improvements in priority order. Each recommendation should say what to change and why. When useful, suggest replacement wording, restructured phases, sharper exit criteria, narrower or broader TDD slices, merged micro-slices, split oversized slices, or additional tasks.

### Open Questions

List questions that should be answered to improve confidence in the plan. Only include this section when such questions remain.

## Examples

Input: "Review the delivery plan for the user-auth-flow feature"

Output:
- a structured critique covering each checklist dimension
- feedback on sequencing (e.g., token validation should precede middleware integration)
- assessment of whether all design acceptance criteria are covered by plan tasks
- evaluation of TDD slice quality (e.g., slices in Phase 1 are well-scoped but Phase 3 batches too many tests in a single slice, or Phase 2 is fragmented into micro-slices that should be merged)
- prioritized recommendations such as adding exit criteria to Phase 2, rebalancing TDD slices in Phase 3, and adding a risk entry for third-party OAuth provider downtime

Input: "Critique the plan I just created for the notification system"

Output:
- a review highlighting strengths in phasing and TDD approach
- finding that the Design Reference section is missing, making traceability difficult
- warnings about missing documentation tasks, vague Phase 3 tasks, and an unaddressed open question from the design about message retry limits
- finding that the Definition of Done omits documentation and code review
- concrete suggestions for tightening task descriptions, reordering phases to reduce dependency risk, and adding integration test coverage

## Edge Cases

- **Very short or partial plan**: Do not refuse. Review what exists, identify the most important missing sections, and state the confidence limits caused by missing detail.
- **Mostly good plan with a few weak spots**: Preserve strengths in the review instead of rewriting the whole plan as if it were poor. Keep the critique proportional.
- **Plan without a source design**: Review the plan on its own merits. Note that the absence of a source design limits the ability to check coverage and alignment. Recommend creating a design if the plan covers complex or ambiguous work.
- **Plan that diverges from its source design**: Call out the divergence explicitly — added scope, dropped requirements, or changed assumptions — and recommend reconciling the plan with the design or updating the design to reflect intentional changes.
- **Plan with no testing strategy**: Flag this as a high-severity finding and suggest what types of testing should be specified based on the nature of the work.
- **Plan with TDD slices that are too broad**: Flag slices that batch many tests or cover too much scope. Suggest narrowing each slice to one meaningful behavior or tightly related behavior cluster with a clear red-green-refactor cycle.
- **Plan with TDD slices that are too narrow**: Flag runs of trivial or highly adjacent slices whose boundaries add more workflow overhead than clarity. Suggest merging them into fewer behavior-oriented slices.
- **Plan with artificial phasing**: If the work is simple enough for a single phase but the plan breaks it into many phases, recommend consolidation and explain why fewer phases would be more effective.
- **Plan with no definition of done**: Flag this as a finding and suggest a checklist derived from the design's acceptance criteria plus standard items (tests passing, documentation written, code reviewed).
- **Plan pasted in conversation**: Review the text directly. Do not insist on locating a stored plan artifact when the content is already available.
- **Review drifting into plan creation**: Suggest structural changes and task improvements when helpful, but keep the primary deliverable as critique and revision guidance rather than producing a replacement plan.

## TDD Slice Sizing Review Heuristics

Use these heuristics when judging whether a plan's TDD slices are well-sized.

### Signs slices are too narrow

- multiple adjacent slices touch the same code area and acceptance criterion with only tiny assertion-level differences
- the plan separates behavior that would naturally be tested and implemented together
- several slices appear to exist only because of implementation order, not because of meaningful behavior or risk boundaries
- the likely workflow overhead per slice would be disproportionate to the value of the separation

### Signs slices are about right

- each slice delivers one meaningful behavior or one tightly related behavior cluster
- each slice has a plausible Red, Green, and Refactor loop with localized feedback if something fails
- the slice descriptions are concrete enough to act on without prescribing every tiny assertion as its own slice
- the sequence builds confidence incrementally without unnecessary handoff overhead

### Signs slices are too broad

- one slice spans several loosely related behaviors or acceptance criteria
- a single slice crosses package, subsystem, or architectural boundaries without a strong reason
- many unrelated tests would need to be written before implementation can begin
- failure or review feedback would likely be diffuse and hard to resolve in one iteration

When recommending changes, optimize for slices that are both conceptually coherent and efficient to execute in the TDD workflow.

---

This page was generated from [`.stencila/skills/software-plan-review/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/software-plan-review/SKILL.md).
