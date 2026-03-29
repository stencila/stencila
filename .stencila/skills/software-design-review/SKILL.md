---
name: software-design-review
title: Software Design Review Skill
description: Critically review a software design specification and suggest concrete improvements. Use when the user wants to review, critique, audit, evaluate, or strengthen a design spec, technical specification, feature design, architecture proposal, or implementation plan. Focus on clarity, completeness, correctness, feasibility, tradeoffs, risks, assumptions, and actionable recommendations that improve the design without turning the task into code generation or workflow design.
keywords:
  - software design
  - design review
  - design spec review
  - technical specification
  - architecture review
  - feature design
  - implementation plan
  - tradeoffs
  - risks
  - feasibility
  - requirements quality
  - actionable feedback
  - not code review
allowed-tools: read_file glob grep
---

## Overview

Review an existing software design specification with the goal of making it clearer, more complete, more correct, and more useful for implementation and decision-making. The skill should produce a structured critique that identifies strengths, weaknesses, risks, open questions, and concrete improvements. Prefer critique plus revision guidance over rewriting the full specification unless the user explicitly asks for a rewritten version.

Use this skill when the user already has a design artifact and wants critical review rather than initial drafting. It complements software-design-creation by evaluating a spec after it has been written. Do not use it when the main task is to write production code, review source code, or create workflows or agents.

## Steps

1. Identify the design to review:
   - accept a design name, an existing stored design, or a pasted design specification
   - restate the design's apparent purpose, target users, and problem being solved
   - if the input is fragmented, reconstruct the main proposal before critiquing it
2. Resolve the design artifact:
   - if the full design specification is already present in the conversation, review that text directly and do not force retrieval through design tools
   - if the user refers to a stored design by name or asks to review an existing design, use `glob` with pattern `.stencila/designs/*.md` to locate likely candidates
   - use `read_file` to load the selected stored design before reviewing it
   - if multiple similarly named designs exist, compare the candidates and review the one that best matches the user's request
3. Understand the design before judging it:
   - summarize the proposed system, feature, or change in plain language
   - identify the stated goals, non-goals, scope boundaries, assumptions, constraints, and acceptance criteria if present
   - note any missing context that materially limits confidence in the review
4. Evaluate the design for clarity and structure:
   - check whether the problem, goals, scope, and intended users are clearly stated
   - check whether sections are organized logically and terminology is consistent
   - flag ambiguous wording, undefined terms, implicit assumptions, and places where a developer or stakeholder could misinterpret the design
5. Evaluate completeness:
   - check whether the design covers the core requirements needed for implementation and review
   - assess whether it addresses functional requirements, non-functional requirements, architecture, interfaces, data, dependencies, security, privacy, observability, rollout, migration, and operations when relevant
   - identify missing decisions, missing edge cases, and missing acceptance criteria or success measures
6. Evaluate correctness and internal consistency:
   - look for contradictions between goals, scope, architecture, interfaces, and acceptance criteria
   - flag requirements that do not appear to solve the stated problem or that conflict with stated constraints
   - identify assumptions that seem unsupported, unrealistic, or inconsistent with the rest of the design
7. Evaluate feasibility and tradeoffs:
   - assess whether the proposed approach is plausible given the stated constraints, dependencies, scale, team context, and delivery goals
   - identify technical, product, operational, or organizational risks
   - call out important tradeoffs such as complexity versus speed, flexibility versus simplicity, consistency versus performance, or cost versus reliability
   - note when the design chooses a tradeoff implicitly and should make it explicit
8. Evaluate implementability and actionability:
   - determine whether engineers could reasonably plan and begin implementation from the document
   - assess whether interfaces, responsibilities, and acceptance criteria are specific enough to guide work and testing
   - flag vague statements that should be rewritten as concrete requirements, decisions, or open questions
9. Produce a structured review report:
   - begin with a concise overall assessment
   - list the most important strengths so the user knows what to keep
   - organize issues by severity or priority rather than by minor editorial order
   - provide actionable recommendations that explain what to change and why
   - when useful, suggest replacement wording, additional sections, or sharper acceptance criteria
10. Distinguish facts from uncertainty:
   - clearly label assumptions made during the review
   - separate definite problems from possible risks or questions that need confirmation
   - avoid inventing system facts that are not supported by the design

## Review Checklist

Assess the design against the following dimensions and tailor the depth to the size of the spec.

### Problem and Scope

- Is the problem statement clear and specific?
- Are the intended users, stakeholders, or operators identified?
- Are goals and non-goals explicit and non-overlapping?
- Is in-scope versus out-of-scope work clear?
- Does the design stay focused on the stated problem?

### Requirements and Acceptance Criteria

- Are the important functional requirements present?
- Are non-functional requirements identified where relevant, such as performance, reliability, security, privacy, accessibility, maintainability, and observability?
- Are acceptance criteria or success measures specific and testable?
- Do requirements describe outcomes rather than vague aspirations or implementation tasks?
- Are major edge cases and failure modes considered?

### Architecture and Interfaces

- Is the proposed architecture understandable and appropriately detailed?
- Are major components, responsibilities, and interactions defined?
- Are external systems, APIs, or integration points identified?
- Are important data models, state transitions, or storage decisions explained when relevant?
- Does the design omit architectural detail that implementers would likely need?

### Correctness and Consistency

- Do different sections agree with each other?
- Do the requirements, architecture, and acceptance criteria align?
- Are assumptions explicit and reasonable?
- Are there contradictions, impossible constraints, or missing dependencies?
- Does the proposed solution actually address the stated problem?

### Feasibility and Risk

- Is the design realistic for the stated timeline, constraints, and environment?
- Are key technical and operational risks identified?
- Are rollout, migration, backward compatibility, or operational concerns covered when relevant?
- Are dependencies and external constraints acknowledged?
- Are the hardest parts of the design surfaced rather than hidden?

### Tradeoffs and Decision Quality

- Does the design explain why this approach was chosen?
- Are plausible alternatives or tradeoffs acknowledged when they matter?
- Are cost, complexity, performance, usability, and maintainability implications visible?
- Are any important decisions left implicit when they should be explicit?
- Does the design over-engineer or under-specify the solution?

### Actionability

- Could a team use this document to estimate, plan, and implement the work?
- Are open questions clearly separated from settled decisions?
- Are next changes to the spec obvious from the review?
- Is the feedback concrete enough to revise the design efficiently?

## Report Format

Structure the review as follows.

### Overall Assessment

One to three sentences summarizing the design's current quality and the most important improvement needed.

### Strengths

A short bullet list of what the design already does well.

### Findings

Group findings under these headings when relevant:

- **Clarity and structure**
- **Completeness**
- **Correctness and consistency**
- **Feasibility and risks**
- **Tradeoffs and decision quality**
- **Actionability**

For each finding:

- indicate severity as **High**, **Medium**, or **Low**
- describe the issue precisely
- explain why it matters

### Recommendations

Provide a numbered list of concrete improvements in priority order. Each recommendation should say what to change and why.

### Open Questions

List questions that should be answered to improve confidence in the design, only when such questions remain.

## Examples

Input: "Review the design for a feature that schedules social media posts"

Output:
- a structured critique of the scheduling design's scope, requirements, architecture, and risks
- feedback on timezone handling, platform validation, failure recovery, and acceptance criteria
- prioritized recommendations such as clarifying supported platforms, specifying retry behavior, and tightening observability requirements

Input: "Please critique our internal tool spec for tracking lab equipment usage"

Output:
- a review highlighting strengths in workflow coverage and data modeling
- warnings about missing auditability, permission rules, and operational assumptions
- concrete suggestions for rollout, reporting requirements, and measurable acceptance criteria

## Edge Cases

- **Very short or partial design**: Do not refuse. Review what exists, identify the most important missing sections, and state the confidence limits caused by missing detail.
- **Mostly good design with a few weak spots**: Preserve strengths in the review instead of rewriting the whole design as if it were poor.
- **Highly speculative design**: Distinguish conjectural risks from confirmed issues and recommend validation steps.
- **Conflicting requirements**: Call out the conflict explicitly, explain the tradeoff, and suggest one or more resolution paths.
- **No acceptance criteria**: Flag this clearly and suggest candidate criteria or the dimensions they should cover.
- **Review drifting into implementation**: Suggest implementation-relevant clarifications when helpful, but keep the primary deliverable as critique and design improvement guidance rather than source code.
