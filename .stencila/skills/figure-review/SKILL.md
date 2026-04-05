---
name: figure-review
description: Critically review an existing or proposed Stencila figure artifact for structural correctness, caption quality, layout, overlay annotation safety, cross-references, measurement validity, and approval readiness. Use when asked to review, critique, assess, audit, validate, or improve a figure block, multi-panel figure, subfigure grid, executable chart figure, caption, SVG overlay, ROI annotation, scale bar, panel labeling, or figure plan.
keywords:
  - review figure
  - critique figure
  - audit figure
  - assess figure
  - validate figure
  - figure caption
  - figure layout
  - subfigure review
  - multi-panel figure
  - executable figure
  - plot review
  - overlay review
  - annotation review
  - SVG overlay
  - cross-reference
  - panel labeling
  - scale bar
  - ROI
  - figure approval
allowed-tools: read_file glob grep snap ask_user
---

## Overview

Review an existing or proposed Stencila figure artifact for correctness, clarity, and approval readiness. The artifact may be a full `.smd` document, a single figure block, a patch or diff, a caption draft, an overlay snippet, a rendered screenshot, or a sufficiently concrete figure plan.

This is a **review-only skill**. Its primary mode is **assess and report**: identify concrete issues, risky assumptions, missing verification, and approval blockers. Do not create a brand-new figure from scratch by default. If the user explicitly asks for a corrective example after the review, keep it minimal and tightly scoped to the findings you already reported.

Use the local references in this skill directory when you need condensed syntax or review guidance:

- [`references/figure-structure-review.md`](references/figure-structure-review.md) — figure fences, captions, subfigures, layouts, and cross-reference checks
- [`references/overlay-review-rules.md`](references/overlay-review-rules.md) — overlay component checks, measurement safety, and rendering risks
- [`references/snap-tool.md`](references/snap-tool.md) — full `snap` tool reference for visual verification

## Required Inputs

| Input | Required | Description |
|---|---|---|
| Figure artifact to review | Required | A `.smd` document, figure block, patch, diff, screenshot, rendered route, or figure plan |
| Review goal | Required | What the user wants assessed: correctness, clarity, approval readiness, caption quality, overlay placement, etc. |
| Surrounding document context | Optional | Nearby text, references, related figures, or house style conventions |
| Rendering access | Optional | A served route or other way to visually inspect the rendered figure with `snap` |
| Calibration or orientation facts | Optional | Known scale-bar calibration, measurements, or compass semantics needed to judge scientific correctness |
| Acceptance criteria | Optional | Standards such as journal style, publication constraints, or project-specific figure conventions |

When used standalone, these inputs come from the user or the agent's prompt. When used within a workflow, the workflow's stage prompt will specify how to obtain them.

## Outputs

| Output | Description |
|---|---|
| Figure review report | Evidence-based findings on structure, captioning, layout, annotations, and approval readiness |
| Prioritized issues | Findings organized by severity such as blocking, important, and optional |
| Verification status | What was checked statically, what was visually verified with `snap`, and what remains pending |
| Minimal corrective example | Optional small snippet only when explicitly requested after the review |

## Core rules

- Read the artifact before judging it. If a full document is available, inspect the surrounding context rather than reviewing an isolated figure in a vacuum.
- Distinguish **static review** from **rendered review**. Do not claim layout or placement problems unless they are visible in the source or were confirmed by `snap`.
- Do not invent scale, measurements, orientation semantics, or annotation coordinates.
- Treat automatic subfigure labels as built-in behavior. Do not require overlay badges for A/B/C panel letters unless the figure needs separate in-image labels for a different purpose.
- Prefer evidence-backed findings over stylistic preference. Cite the exact caption text, snippet, layout pattern, annotation, or rendered result behind each finding.
- When the input is only a vague aspiration such as “make the figure more compelling,” say that there is not yet enough concrete material for implementation review and either review it as an early direction only or ask for more specificity.

## Review lens

Evaluate the artifact against these dimensions:

1. **Concreteness**
   - Is there enough material to review now: file, snippet, patch, screenshot, or a plan with concrete panel content and caption intent?
   - If the input is only a loose idea, say that it is not ready for implementation review yet.
2. **Figure structure and syntax**
   - Are `::: figure` fences used correctly?
   - Does the figure separate content and caption cleanly?
   - If subfigures are used, are nested `::: figure` blocks indented by four spaces?
   - If a layout mini-language is used, is it syntactically coherent and consistent with the number/order of subfigures?
3. **Caption and narrative fit**
   - Does the caption describe the figure accurately and at the right level of detail?
   - Is panel order clear and consistent with the caption text?
   - Does the caption avoid making claims the visual content does not support?
4. **Cross-references and labeling**
   - Do figure references point to the right target?
   - Are automatic figure and subfigure labels sufficient, or has the author accidentally duplicated them with overlay badges?
   - Are IDs, references, and nearby mentions consistent with the document context?
5. **Overlay and annotation quality**
   - Are overlays scoped correctly: subfigure-local, parent-level, or both?
   - When using `<s:*>` components, is `xmlns:s="https://stencila.io/svg"` declared?
   - Does the `viewBox` define a coherent coordinate system for all annotations?
   - Are annotations understandable, minimally cluttered, and semantically appropriate?
   - Would anchor-based positioning make the overlay less brittle than repeated raw coordinates?
6. **Measurement and scientific safety**
   - Are scale bars calibrated from known information rather than guessed?
   - Are dimension lines or angle labels supported by known measurements?
   - Are compass indicators used only when orientation semantics are meaningful and known?
7. **Rendering and responsiveness**
   - If the figure is executable and overlaid, do chart dimensions and overlay coordinates appear aligned?
   - If a parent overlay spans a multi-panel grid, is there a risk it becomes misleading when the layout collapses on small screens?
   - Were these concerns verified visually with `snap`, or are they pending?
8. **Approval readiness**
   - Is the figure ready to approve as-is, acceptable with minor revisions, or not ready?
   - What concrete fixes or follow-up verification would move it to approval?

## Visual verification with `snap`

Use `snap` when a Stencila server and route are available. `snap` is the evidence source for rendered placement, spacing, clipping, and annotation overlap.

### When to use `snap`

- **At the start of review** when the user wants rendered QA rather than source-only review
- **To verify a specific concern** such as clipped captions, misaligned overlays, overlapping callouts, or panel-order confusion
- **After reviewing source** to confirm whether a suspicious pattern is actually a visible defect

### Typical `snap` checks for figure review

1. Overall figure capture: `snap(route: "/docs/", selector: "stencila-figure", screenshot: true)`
2. Focus a specific figure when the route has many figures by using a narrower selector or reviewing one figure at a time
3. Re-snap after a proposed correction only if the environment supports it and the figure has actually been updated

Prefer the rendered directory route when the source file is `index.*`, `main.*`, or `README.*`; for example, `docs/README.md`, `docs/main.md`, and `docs/index.md` all render at `"/docs/"`.

### When `snap` is unavailable

If `snap` cannot be run, do not fabricate rendered findings. Instead:

- State that visual verification is **pending** and explain why
- Limit findings to source-visible issues and clearly mark layout/placement concerns as unverified risks
- Recommend the exact `snap` command the user should run once the route is available

## Steps

1. **Identify the review input and desired output.**
   - Determine whether the user wants a quick critique, a full approval review, caption review, overlay review, or revision guidance.
   - Determine whether the artifact is a full document, a single figure block, a patch, a screenshot, or a plan.
   - If the requested review depends on a missing artifact, ask for it.

2. **Read the surrounding context before judging details.**
   - If a document is available, inspect nearby paragraphs, adjacent figures, and figure references.
   - Review the author’s existing caption tone, panel ordering conventions, and cross-reference style.
   - Do not review a patch in isolation when the surrounding figure or document context is available and materially affects the judgment.

3. **Classify the figure type and review scope.**
   - Determine whether the artifact is a simple image figure, executable figure, multi-panel figure, or any of these with overlays.
   - Determine whether the request is mostly about structure, captioning, annotations, rendering, or scientific correctness.

4. **Check structure and syntax first.**
   - Use [`references/figure-structure-review.md`](references/figure-structure-review.md) to confirm fence structure, caption placement, subfigure indentation, and layout usage.
   - Flag malformed fences, missing indentation, caption/content confusion, or layouts that do not match the panel count.
   - If the input is only a rendered screenshot and not source, say which syntax checks could not be performed.

5. **Review caption, panel order, and references.**
   - Check whether the caption accurately describes what the figure shows.
   - Check whether panel references in the caption match the visual order.
   - Check whether figure references in surrounding text appear to target the correct figure.
   - Flag duplicate manual panel lettering when automatic subfigure labels already cover the same purpose.

6. **Review overlays and annotations.**
   - Use [`references/overlay-review-rules.md`](references/overlay-review-rules.md) to assess component usage, scope, clutter, and coordinate coherence.
   - Flag missing `xmlns:s`, inconsistent `viewBox` assumptions, unclear annotation targets, and overly brittle coordinate repetition.
   - Prefer actionable findings such as “replace repeated raw coordinates with a named anchor” over generic comments such as “clean this up.”

7. **Check measurement safety explicitly.**
   - Treat guessed scale bars, invented dimensions, unsupported angle labels, and unjustified compass indicators as serious review findings.
   - If calibration or orientation information is absent, recommend asking for it rather than approving the annotation.

8. **Visually verify with `snap` when possible.**
   - Use `snap` to confirm source concerns that depend on rendering.
   - Check for caption overflow, label collisions, cropped overlays, overlapping callouts, and visually misleading parent-level overlays on multi-panel figures.
   - If `snap` was not run, keep the review honest about what remains unverified.

9. **Separate findings by severity and evidence.**
   - Use severity levels such as:
     - **Blocking** — likely incorrect, misleading, unverified in a material way, or structurally broken
     - **Important** — should be fixed for clarity, maintainability, or likely rendering correctness
     - **Optional** — polish, simplification, or future improvement
   - For each finding, cite the relevant snippet, caption phrase, panel, annotation, or rendered behavior.

10. **Produce the review in the right shape.**
    - For a quick critique, give a short verdict plus the top few findings.
    - For a full review, provide:
      1. figure type and scope reviewed
      2. artifact reviewed
      3. what is already strong
      4. blocking findings
      5. important findings
      6. optional improvements
      7. verification status (`snap` results or pending)
      8. final verdict or recommended next step

11. **Keep corrective authoring secondary and explicitly requested.**
    - Primary mode is assess-and-report.
    - Only after completing the review, and only if the user explicitly asks, provide a minimal corrective snippet.
    - Keep any example tightly scoped to the findings you already reported.

## Output guidelines

When useful, structure the response like this:

1. Figure type and scope reviewed
2. Artifact reviewed
3. What is already strong
4. Blocking findings
5. Important findings
6. Optional improvements
7. Verification status
8. Final verdict or next step

Each finding should be evidence-based. Use source snippets and `snap` results as evidence when available. Do not make speculative claims about rendering, measurements, or scientific semantics.

## Examples

### Example 1: review a multi-panel figure with overlays

Input: Review this `.smd` figure for publication readiness. I am worried about panel labels and overlay clutter.

Output:

1. Figure type and scope reviewed: multi-panel image figure with per-panel overlays
2. Artifact reviewed: one parent `::: figure [2]` block with two subfigures and SVG overlays
3. What is already strong:
   - panel structure is clear and the two-column layout matches the visual comparison
   - captions for each panel are concise and informative
4. Blocking findings:
   - none
5. Important findings:
   - panel A and B each include `<s:badge label="A">` / `<s:badge label="B">` even though subfigure labels are already automatic, so the rendered figure will likely show duplicate lettering
   - the left overlay repeats the same target coordinates in three annotations; replace them with a named anchor so later panel edits do not break all annotations
6. Optional improvements:
   - shorten the parent caption’s opening sentence so the key comparison appears earlier
7. Verification status:
   - source reviewed
   - visual verification pending because no served route was provided
8. Final verdict or next step:
   - acceptable with revisions before approval

### Example 2: review a scale bar and compass annotation

Input: Audit this microscopy figure for annotation correctness.

Output:

1. Figure type and scope reviewed: single image figure with overlay annotations
2. Artifact reviewed: image figure plus `<s:scale-bar>` and `<s:compass>` components
3. What is already strong:
   - annotations are visually restrained and easy to interpret
4. Blocking findings:
   - the scale bar label says `20 μm` but the artifact provides no calibration linking viewBox units or pixels to real distance, so the measurement is unsupported and should not be approved as-is
   - the compass uses `N/S E/W` semantics with no evidence that geographic orientation is meaningful for this image
5. Important findings:
   - none
6. Optional improvements:
   - if orientation is meaningful, replace the generic compass with axis labels that match the domain-specific semantics
7. Verification status:
   - source reviewed
   - rendered placement not checked with `snap`
8. Final verdict or next step:
   - not ready to approve until calibration and orientation semantics are confirmed

### Example 3: review an executable figure with overlay alignment risk

Input: Critique this Plotly figure with annotations.

Output:

1. Figure type and scope reviewed: executable chart figure with SVG overlay
2. Artifact reviewed: Plotly code block plus `svg overlay`
3. What is already strong:
   - the caption explains the chart clearly and the overlay intent is appropriate
4. Blocking findings:
   - none
5. Important findings:
   - the Plotly layout does not set explicit `width` and `height`, so responsive resizing may cause the overlay `viewBox` to drift out of alignment with the chart
   - the callout target is plausible in source but requires rendered verification
6. Optional improvements:
   - use an anchor for the peak annotation rather than repeating the same coordinates across the halo and callout
7. Verification status:
   - static review complete
   - run `snap(route: "/docs/", selector: "stencila-figure", screenshot: true)` to confirm overlay alignment
8. Final verdict or next step:
   - acceptable with revisions and visual verification

## Edge cases

- **Only a screenshot is provided**: review visual clarity and likely issues, but clearly mark all source-level syntax findings as unavailable without the `.smd`.
- **Only a caption is provided**: review caption accuracy, ambiguity, panel-order wording, and narrative fit, but do not make source or rendering claims you cannot support.
- **Patch without surrounding context**: say what can be reviewed from the patch alone and what requires the full figure or document.
- **Subfigure badges intentionally differ from panel labels**: do not flag them as duplicates if the badges encode different semantics than A/B/C panel lettering.
- **No calibration data**: treat scale bars and dimension lines as unverified rather than guessing whether they are correct.
- **No running Stencila route**: keep visual verification pending and recommend the exact `snap` command to run later.
- **User asks for fixes instead of review**: provide the review first; only then provide a minimal corrective example if explicitly requested.

## Limitations

- This skill reviews figure structure, clarity, and likely rendering correctness. It does not prove scientific truth beyond the evidence supplied.
- The review does not execute external analysis pipelines or infer hidden measurements from images.
- Without `snap`, rendering-related findings remain static-analysis judgments rather than visually confirmed defects.
