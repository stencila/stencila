---
name: software-design-creation
description: Create software design specifications for new features or standalone software. Use when the user needs help turning a brief idea into a concrete design spec by eliciting missing requirements, clarifying scope, defining users and constraints, outlining architecture and interfaces, identifying risks and assumptions, and writing acceptance criteria that developers can later use for planning and implementation.
keywords:
  - software design
  - design spec
  - software specification
  - technical specification
  - feature design
  - feature requirements
  - product requirements
  - requirements gathering
  - acceptance criteria
  - architecture
  - implementation planning
  - not code review
allowed-tools: read_file write_file edit_file apply_patch glob grep ask_user
---

## Overview

Create a software design specification that developers can use to plan and implement a feature or standalone software system. The skill should expand a minimal or ambiguous prompt into a structured design artifact, asking clarifying questions when important requirements are missing.

Use this skill when the user wants a design spec, technical plan, feature specification, architecture outline, or implementation-ready requirements document. Do not use it when the main task is to write production code, review code, or create a workflow or agent.

## Steps

1. Identify the design target:
   - determine whether the request is for a new feature, a product change, or a standalone software system
   - restate the problem in plain language
   - identify the intended users, operators, or stakeholders if known
2. Assess whether the prompt is complete enough to draft a useful spec:
   - if critical information is missing, ask concise clarifying questions before writing the full spec
   - prioritize questions about user goals, scope boundaries, constraints, integrations, platforms, data, security, and success conditions
   - if the interaction is one-shot or back-and-forth is limited, ask at most a few critical questions and then proceed with explicit assumptions rather than blocking on missing answers
   - if the user does not know all answers, record reasonable assumptions explicitly instead of blocking progress
3. Convert the request into a fuller design brief:
   - summarize the problem being solved
   - describe the user need or business objective
   - distinguish core requirements from optional enhancements
   - define what is in scope and out of scope
4. Draft the design specification with the sections that fit the task:
   - title and summary
   - background or context
   - goals and non-goals
   - users or actors
   - use cases or user flows
   - functional requirements
   - non-functional requirements such as performance, reliability, accessibility, security, privacy, compliance, observability, and maintainability
   - system design or architecture overview
   - components, services, and responsibilities
   - data model or important entities when relevant
   - APIs, interfaces, or integration points when relevant
   - dependencies, assumptions, constraints, and risks
   - rollout or migration considerations when relevant
5. Make the design actionable for implementation:
   - write acceptance criteria that are specific, testable, and tied to user-visible or system-visible outcomes
   - ensure acceptance criteria describe observable behavior, interfaces, or system qualities rather than implementation tasks
   - separate must-have criteria from optional follow-up work when useful
   - note open questions and decisions still needed
   - include suggested implementation slices or milestones only when they help clarify delivery
6. Tailor the level of detail to the user request:
   - for a vague idea, produce a lightweight but structured spec with explicit assumptions
   - for a mature request, provide a more complete design with clearer architecture, interfaces, and constraints
   - avoid inventing unnecessary complexity
7. Before finishing, check the spec quality:
   - ensure the document is internally consistent
   - ensure acceptance criteria are observable and not vague
   - ensure assumptions and unknowns are clearly labeled
   - ensure the output supports downstream planning and implementation
8. Always persist the completed design:
   - write the design to `.stencila/designs/{name}.md` using `write_file`, where `{name}` is a concise kebab-case name for the design (e.g. `user-auth-flow`)
   - create the `.stencila/designs/` directory if it does not exist (the `write_file` tool creates parent directories automatically)
   - for updates to an existing design, prefer `edit_file` or `apply_patch` over rewriting the entire file

## Suggested Output Structure

Use a structure like this when appropriate. Omit sections that do not apply and add sections that materially improve the design.

```markdown
# <feature or system name>

## Summary

A concise description of the feature or software and the value it delivers.

## Problem

What problem is being solved and for whom.

## Goals

- ...

## Non-goals

- ...

## Users / Actors

- ...

## Scope

### In scope

- ...

### Out of scope

- ...

## Requirements

### Functional requirements

- ...

### Non-functional requirements

- ...

## Design

### Architecture overview

- ...

### Components

- ...

### Data / Interfaces

- ...

## Risks and Assumptions

- ...

## Open Questions

- ...

## Acceptance Criteria

- Given / when / then style criteria, bullet criteria, or numbered criteria
- Criteria should be testable and implementation-relevant
```

## Clarifying Questions

When the prompt is underspecified, ask only the highest-value questions first. Prefer concise questions such as:

1. Who are the primary users and what job are they trying to do?
2. What must the feature or system do on day one?
3. What is explicitly out of scope?
4. Are there platform, technology, integration, or regulatory constraints?
5. What quality requirements matter most: performance, reliability, security, accessibility, privacy, or cost?
6. How will success be measured?
7. Are there existing systems, APIs, or data models this must work with?

If the user cannot answer everything, proceed with explicit assumptions.

## Examples

Input: "Design a feature for scheduling social media posts"

Output:
- a feature design spec that identifies target users such as social media managers
- a scoped requirement set for creating, editing, previewing, and scheduling posts
- relevant constraints such as timezone handling and platform API limits
- acceptance criteria for scheduling, editing, validation, and failure handling

Mini spec excerpt:

```markdown
# Social Media Post Scheduling

## Summary

Enable social media managers to compose posts, preview them, and schedule publication to supported platforms at a future date and time.

## Goals

- let users schedule a post for a specific platform, date, time, and timezone
- prevent invalid or incomplete scheduled posts from being saved
- surface delivery failures clearly so users can retry or edit the post

## Non-goals

- creating new social media platform integrations beyond the supported set
- advanced campaign analytics beyond basic delivery status

## Functional requirements

- users can create, edit, reschedule, and cancel scheduled posts before publication
- the system validates required content and platform-specific constraints before saving
- the system stores the intended publish time with an explicit timezone

## Acceptance Criteria

1. Given a user enters valid post content and a future publish time, when they save the schedule, then the system stores the scheduled post and shows it in the upcoming posts list.
2. Given a user selects a publish time in their local timezone, when the schedule is saved, then the system preserves the timezone and displays the equivalent scheduled time consistently on reload.
3. Given a post violates a platform rule such as character limit, when the user attempts to schedule it, then the system blocks the action and shows a specific validation message.
4. Given a publish attempt fails because of an external platform API error, when the failure is detected, then the system marks the post as failed and provides a visible retry path.
```

Input: "We need a lightweight internal tool for tracking lab equipment usage"

Output:
- a standalone software design spec covering users, workflows, data entities, reporting needs, and operational constraints
- a pragmatic architecture outline suitable for internal deployment
- acceptance criteria that let developers plan implementation and testing

## Edge Cases

- **Very short prompt**: Do not refuse. Ask a few targeted questions, then draft a spec with assumptions if answers are unavailable.
- **User wants implementation details immediately**: Provide enough architecture and interface detail to support implementation, but keep the primary deliverable as a design spec rather than source code.
- **Conflicting requirements**: Call out the conflict explicitly, document tradeoffs, and propose a recommended resolution.
- **Unclear scope**: Split the design into must-have scope and deferred scope instead of producing a fuzzy all-in-one spec.
- **Missing non-functional requirements**: Add a short section of recommended defaults and label them as assumptions.
- **Acceptance criteria drifting into project process**: Keep criteria focused on what must be true of the delivered feature or system, not on team ceremonies or workflow steps.
