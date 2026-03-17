---
name: software-delivery-full
description: End-to-end software feature pipeline that composes design, planning, and TDD delivery workflows into a single orchestrated sequence
goal-hint: What feature or capability do you want to design, plan, and deliver?
keywords:
  - software
  - feature
  - lifecycle
  - end-to-end
  - design
  - planning
  - delivery
  - tdd
when-to-use:
  - when a feature needs to go from idea through design, planning, and implementation in one pipeline
  - when you want the full design-plan-deliver lifecycle orchestrated automatically with human approval gates at each stage
  - when starting from scratch with a feature idea and wanting TDD-based delivery of the result
when-not-to-use:
  - when a design specification already exists and only planning and delivery are needed
  - when a delivery plan already exists and only TDD execution is needed
  - when you want to run design, planning, or delivery independently with different goals
  - when the task does not involve software implementation
---

This workflow composes three child workflows into a sequential end-to-end pipeline for taking a software feature from idea to working code:

1. **Design** — runs `software-design-iterative` to draft and refine a design specification through agent review and human approval
2. **Plan** — runs `software-plan-iterative` to create and refine a delivery plan from the approved design through agent review and human approval
3. **Deliver** — runs `software-delivery-tdd` to execute the delivery plan slice-by-slice using Red-Green-Refactor TDD cycles with human sign-off after each slice

Each child workflow contains its own internal review and refinement loops, so this parent workflow focuses purely on orchestration: passing the user's goal through each stage in sequence. The user provides their feature idea once, and each composed workflow receives it as `$goal` to drive its internal agents.

```dot
digraph software_feature_lifecycle {
  Start -> Design

  Design [workflow="software-design-iterative", label="Design specification"]
  Design -> Plan

  Plan [workflow="software-plan-iterative", label="Delivery plan"]
  Plan -> Deliver

  Deliver [workflow="software-delivery-tdd", label="Test-driven delivery"]
  Deliver -> End
}
```
