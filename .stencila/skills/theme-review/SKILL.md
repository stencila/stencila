---
name: theme-review
description: Critically review an existing or proposed Stencila theme artifact for correctness, token usage, target coverage, cross-target portability, dark-mode handling, maintainability, and approval readiness. Use when asked to review, critique, assess, audit, or validate a theme.css file, theme patch, theme plan, site theme, document theme, plot theme, print or PDF theme, check design tokens, assess DOCX or email behavior, review dark mode support, or validate with stencila themes validate.
keywords:
  - review theme.css
  - critique theme
  - audit theme
  - assess theme
  - review document theme
  - review site theme
  - review pdf theme
  - review docx theme
  - review email theme
  - review plot theme
  - review navigation theme
  - review dark mode
  - design tokens
  - css custom properties
  - theme validation
  - cross-target portability
  - token correctness
  - theme approval
allowed-tools: read_file glob grep shell
---

## Overview

Review an existing or proposed Stencila theme artifact for correctness, token usage, target coverage, portability, maintainability, and approval readiness.

The artifact may be a workspace `theme.css`, a patch or diff, a CSS snippet, or a sufficiently concrete theme plan before implementation. The primary mode of this skill is **assess and report**: identify concrete issues, missing verification, risky assumptions, approval blockers, and target-specific gaps.

This is a **review-only skill**. Do not use it to create a new skill, write `SKILL.md`, design a theme from scratch, or generate a full new theme by default. Use it when the user wants an assessment of an existing artifact or a concrete proposed direction.

Use the local references in this skill directory when you need architecture or token-family guidance:

- [`references/themes-guide.md`](references/themes-guide.md) for theme architecture, resolution rules, and cross-target constraints
- [`references/semantic-and-font-tokens.md`](references/semantic-and-font-tokens.md) for semantic-token and font review guidance
- [`references/node-token-families.md`](references/node-token-families.md) for document/node token families
- [`references/print-and-pdf-tokens.md`](references/print-and-pdf-tokens.md) for print and PDF page tokens
- [`references/plot-tokens.md`](references/plot-tokens.md) for plot theming review
- [`references/site-token-families.md`](references/site-token-families.md) for site-theme component families and naming quirks
- [`references/cli-commands.md`](references/cli-commands.md) for token discovery and validation commands

Also use the theme CLI as the live source of truth when available:

- `stencila themes` or `stencila themes list` to list all available themes (workspace, user, and builtin) with their type and location
- `stencila themes show [NAME]` to display the resolved CSS of a theme (omit the name for the default resolved theme); add `--verbose` to also show resolved CSS variable values — especially useful for understanding what values are inherited from the base theme
- `stencila themes tokens` to list builtin tokens, optionally filtered by `--scope` (`semantic`, `node`, `site`, `plot`, `print`) and `--family`, with `--as json|yaml|toml` for machine-readable output
- `stencila themes validate <FILE>` to check that a CSS theme parses and that custom properties correspond to known builtin design tokens; use `--strict` when unknown tokens should fail validation

Stencila themes are token-first. In review, prefer semantic tokens as the stable public API, expect module-specific tokens only where needed, and flag broad custom CSS rules when tokens would be clearer, safer, or more portable.

## Dark mode review

Many tokens have `*-dark` variants (e.g., `--text-color-primary-dark`, `--surface-background-dark`, `--plot-background-dark`). The base theme applies these automatically via `prefers-color-scheme: dark`. When reviewing a theme:

- Check whether the theme sets both light and `*-dark` variants when colors differ between schemes.
- Flag cases where accent, surface, or text colors are set for light mode only but would look wrong on a dark background.
- Check dark variants for plot, surface, and text tokens especially — colors that work on a light background often need adjustment for dark backgrounds.
- Use `stencila themes tokens --scope semantic` or `--scope plot` to see which tokens have dark variants.
- Dark variants are only relevant for web and HTML-derived outputs; non-web targets such as DOCX and email do not use dark mode. Flag claims of dark-mode DOCX or email support as incorrect.

## Review lens

Evaluate the artifact against these dimensions:

1. Concreteness
   - Is there enough material to review now: file, patch, snippet, or a plan with concrete tokens, surfaces, and targets?
   - If the input is only a vague aspiration such as “make it calmer and more premium,” say that it is not ready for implementation review yet and either review it as an early direction only or ask for more specificity.
2. Correct target fit
   - Is this a document theme, site theme, plot theme, print/PDF theme, or a mixed theme?
   - Do the chosen tokens and selectors match the stated surfaces?
3. Token correctness
   - Are semantic tokens used where possible?
   - Are module-specific tokens appropriate and, where exact names matter, verified?
   - Are there duplicated, conflicting, or shadowed token assignments?
4. Cross-target portability
   - Are exportable values kept in top-level `:root` (i.e., not nested inside `@media`, `@supports`, or other at-rules)?
   - Are web-only constructs being mistaken for cross-target theming?
   - Are DOCX, email, PDF, Python, and R caveats acknowledged where relevant?
5. Maintainability and scope
   - Are selectors focused and minimal?
   - Is the patch or plan scoped to the user’s request?
   - Does the artifact preserve existing imports, asset paths, and unrelated styling when patching?
6. Validation and approval readiness
   - Should `stencila themes validate <FILE>` or `--strict` be run?
   - Should exact names be checked with `stencila themes tokens --scope ... --family ...`?
   - Is the artifact ready to approve, acceptable with minor issues, or not ready to approve?

## Base theme loading

Stencila automatically loads `base.css` before any theme CSS — both when rendering HTML (as a separate `<link>` tag) and when computing theme variables (base variables are merged before theme-specific ones). Users do not have `base.css` in their workspace.

- Do not treat the absence of `@import url("./base.css")` as a problem.
- Treat the **presence** of `@import url("./base.css")` in a workspace theme as a review finding — it is unnecessary and may cause issues.
- Expect user themes to start directly with optional external font imports and `:root` overrides.

## Steps

1. Identify the review input and desired output.
   - Determine whether the user wants a quick critique, a full review, prioritized findings, an approval check, or revision guidance.
   - Determine whether the artifact is a plan, full `theme.css`, patch, snippet, or a verbal proposal.
   - If the artifact is missing and the request depends on it, ask for the file, diff, or snippet.
   - If the input is only a vague design aspiration, say that there is not yet enough concrete material for implementation review and either review it as a direction-setting plan or ask for more specificity.

2. Classify the theme type and targets.
   - Determine whether the artifact concerns a document theme, site theme, plot theme, print/PDF theme, or several outputs.
   - If the target surfaces are unclear and that ambiguity changes the review materially, ask a clarifying question.

3. Inspect the existing context before judging changes.
   - If files are available, inspect the current `theme.css` and related assets before reviewing a proposed patch.
   - Use `stencila themes show` to see the current default resolved theme CSS, or `stencila themes show <NAME>` for a specific theme. Add `--verbose` to also see resolved variable values — this is especially useful for understanding what values the theme inherits and what overrides actually change.
   - Use `stencila themes list` to see all available themes (workspace, user, and builtin) and their locations.
   - Compare the requested changes against the existing token vocabulary, selectors, and import style.
   - When reviewing a patch, check whether it preserves unrelated styling and modifies only the relevant areas.
   - Keep Stencila resolution order in mind when commenting on file choice: when no theme is specified, Stencila resolves workspace `theme.css` first while walking up from the document path, then user `default.css`, then builtin `stencila.css`.
   - If the user refers to a named theme, remember that named themes are resolved from user themes first, then builtin themes.

4. Check the semantic foundation first.
   - Use [`references/semantic-and-font-tokens.md`](references/semantic-and-font-tokens.md) to assess whether the artifact starts with stable semantic tokens before dropping to module-specific tokens or selectors.
   - Flag cases where typography, color, surface, spacing, width, or border concerns are solved with brittle selectors instead of clear semantic tokens.
   - Treat primitive font-stack edits as an advanced move, not the default recommendation.

5. Verify exact module-specific tokens when they matter.
   - Do not approve guessed exact names when a concrete implementation depends on them.
   - Use the CLI first for comprehensive and current inventories:
     - `stencila themes tokens`
     - `stencila themes tokens --scope semantic`
     - `stencila themes tokens --scope site --family nav-menu`
     - `stencila themes tokens --scope plot --as json`
   - Then use the local references for application guidance:
     - semantic and fonts: [`references/semantic-and-font-tokens.md`](references/semantic-and-font-tokens.md)
     - node/document families: [`references/node-token-families.md`](references/node-token-families.md)
     - print and PDF: [`references/print-and-pdf-tokens.md`](references/print-and-pdf-tokens.md)
     - plots: [`references/plot-tokens.md`](references/plot-tokens.md)
     - site families: [`references/site-token-families.md`](references/site-token-families.md)
   - If a name is not verified, recommend verifying it instead of treating it as correct.

6. Review by theme type.
   - **Document theme**: check whether semantic tokens cover most needs, then assess node/module token families for headings, paragraphs, lists, links, code, tables, figures, citations, references, plots, and print/page behavior.
   - **Site theme**: check document foundations plus site families such as `layout`, `nav-menu`, `nav-tree`, `nav-groups`, `breadcrumbs`, `toc-tree`, `prev-next`, `logo`, `title`, and `site-search`.
   - **Plot theme**: ensure explicit `--plot-*` tokens are used and call out non-transferable CSS.
   - **Print/PDF theme**: ensure exportable page tokens remain in top-level `:root` and review page-fit, margin, and header/footer assumptions.

7. Check dark-mode handling.
   - If the theme sets light-mode colors, check whether corresponding `*-dark` variants are defined when the values would not work on a dark background.
   - Flag missing dark variants for accent, surface, text, and plot tokens when the theme is used on web targets.
   - Do not require dark variants for tokens that are inherently scheme-neutral (e.g., content width, spacing, font families).
   - If the theme claims dark-mode support for non-web targets, flag that as incorrect.

8. Flag cross-target risks explicitly.
   - Note when tokens that should affect DOCX, email, PDF, and plots are not defined at top-level `:root`.
   - Flag reliance on `@media`, `@supports`, or browser-only selectors for values that the user expects to export across targets.
   - State plainly when parity is approximate rather than guaranteed.
   - If the artifact affects plots, ensure Python/R transfer is reviewed separately from web rendering.

9. Separate findings by severity and evidence.
   - Label issues as one of:
     - **Blocking**: likely incorrect, invalid, unverifiable, or materially misaligned with the request
     - **Important**: should be fixed for portability, maintainability, or likely target behavior
     - **Optional**: polish, simplification, or future improvement
   - For each finding, cite the relevant token, selector, file area, snippet, or target assumption.
   - Prefer concise, actionable review guidance over generic criticism.

10. Produce the right review output.
   - For a quick review, provide a short verdict plus the top few findings.
   - For a full review, provide:
     1. theme type and targets reviewed
     2. artifact reviewed
     3. what looks good
     4. blocking findings
     5. important findings
     6. optional improvements
     7. validation and verification commands
     8. final verdict or recommended next step

11. Keep corrections secondary and explicitly requested.
    - Primary mode is assess-and-report.
    - Only after completing the review, and only if the user explicitly asks, provide a minimal corrective example.
    - Keep any example tightly scoped to the findings you already reported.

12. Recommend validation with target-specific checks.
    - When a concrete theme file path exists, recommend or run `stencila themes validate <FILE>` before concluding.
    - Use `stencila themes validate <FILE> --strict` when unknown tokens should fail review.
    - Use `stencila themes validate <FILE> --as json` when machine-readable validation results are useful.
    - Recommend concrete checks for each required target:
      - HTML or site preview for screen behavior, responsive layout, and site chrome styling
      - PDF or print preview for page size, margins, header/footer content, first-page overrides, and page-fit of tables/figures/code blocks
      - DOCX for heading, body, link, table, figure-caption, and page-layout mapping
      - email output for simplified typography, spacing, surfaces, and safe component styling
      - Python/R execution for `--plot-*` token transfer to plots

## Output guidelines

When useful, structure the response like this:

1. Theme type and target(s) reviewed
2. Artifact reviewed (`theme.css`, patch, snippet, or plan)
3. What is already strong
4. Blocking findings
5. Important findings
6. Optional improvements
7. Validation and token-verification commands
8. Final verdict or next step

Each finding should be evidence-based. Do not make speculative claims without citing the token, selector, file region, snippet, or target assumption involved.

## Examples

Input: Review this `theme.css` for a research-report document theme. I care about PDF and DOCX too.

Output:

1. Theme type and target(s) reviewed: document theme for web, PDF, and DOCX
2. Artifact reviewed: full `theme.css`
3. What is already strong:
   - starts from semantic typography and surface tokens in top-level `:root`
   - keeps page tokens at top-level `:root`
   - avoids broad selector rewrites for headings and paragraphs
4. Blocking findings:
   - none
5. Important findings:
   - `theme.css:42-49` uses `table { border-color: #d6dae1; }` as the primary table treatment. Review whether verified table-related tokens are available first, because this selector-level rule is broader and less portable than a token-based override.
   - `theme.css:88-94` sets `--page-margin-top` and related page tokens only inside `@media print`. If PDF and DOCX behavior matters, move those exportable page tokens to top-level `:root`.
6. Optional improvements:
   - add explicit link tokens if link styling should remain stable across targets
7. Validation and token-verification commands:
   - `stencila themes validate theme.css --strict`
   - `stencila themes tokens --scope print`
   - `stencila themes tokens --scope node --family table`
8. Final verdict or next step:
   - good semantic foundation; revise page-token placement and check whether table styling can move from the broad selector to verified table tokens before approval

Input: Please review this patch for our published site theme header and navigation.

Patch excerpt:

```diff
 :root {
   --layout-header-height: 48px;
   --nav-menu-color: #6b7280;
   --nav-menu-color-active: #0f766e;
   --search-modal-width: 34rem;
 }

 header nav a {
   letter-spacing: 0.08em;
 }
```

Output:

1. Theme type and target(s) reviewed: site theme for published-site chrome and document shell
2. Artifact reviewed: patch excerpt
3. What is already strong:
   - the patch is scoped to header, navigation, and search surfaces
   - it uses recognizable site-theme token families for the main changes
4. Blocking findings:
   - none if `--search-modal-width` has been verified for the current build; otherwise verify it with `stencila themes tokens --scope site --family site-search` before approval because the patch depends on that exact name
5. Important findings:
   - the selector `header nav a { letter-spacing: 0.08em; }` is a broad site-chrome override without evidence that a token is insufficient. Prefer verified site tokens first, or justify why this selector is necessary.
   - reducing `--layout-header-height` to `48px` should trigger a responsive review of nav wrapping, focus outlines, and search-trigger fit on narrow viewports.
6. Optional improvements:
   - review whether `--nav-menu-color-active` should align with the broader accent system for consistency
7. Validation and token-verification commands:
   - `stencila themes tokens --scope site --family nav-menu`
   - `stencila themes tokens --scope site --family site-search`
   - `stencila themes validate theme.css`
8. Final verdict or next step:
   - likely acceptable after exact-name verification and a responsive header preview

Input: Create a new theme for our publication site with a premium dark style.

Output:

Do not use this skill. This is a creation request, not a review request. Use the `theme-creation` skill instead.

Input: Review this theme idea: “make it calmer and more premium.” There is no CSS yet.

Output:

Before giving detailed review feedback, say that there is no concrete artifact to review and switch to plan review:

> I can review the direction, but not the implementation yet. Is this for a document theme, a published-site theme, or both, and which outputs matter most: web, PDF, DOCX, email, or Python/R plots?

Then assess whether the stated direction is specific enough to map to semantic tokens, site token families, assets, and validation.

## Edge cases

- If the user asks for review but provides no artifact, review the plan or ask for the missing file, patch, or snippet.
- If the request is to create a theme, design a theme from scratch, or write a new skill, do not use this skill; redirect to the `theme-creation` skill.
- If the user does not specify whether the theme is for documents or published sites, infer document theme only for clearly document-centric requests; otherwise ask a clarifying question.
- If the user asks whether a theme is “good” without naming targets, say the answer depends on targets and review against the most likely one only if the context is strong enough to justify that assumption.
- If the user provides a patch without the current file and correctness depends on surrounding context, inspect the existing file when possible.
- If exact token availability is uncertain, run `stencila themes tokens` with the narrowest useful `--scope` and `--family` filters instead of guessing from memory.
- If the artifact mixes semantic tokens, module tokens, and custom selectors, do not reject it automatically; check whether each layer is justified.
- If the user depends on DOCX or email parity, explicitly call out unsupported, not-yet-mapped, or simplified areas instead of claiming full fidelity.
- If important exported tokens are placed inside `@media` or `@supports`, flag that as a portability issue.
- If the theme references assets that are not present in the workspace, do not fabricate files; note the missing assets as a review dependency.
- If the user asks for a corrected snippet or patch after the review, keep the fix scoped to the evidence-based findings already reported.
- If the theme includes `@import url("./base.css")`, flag it as unnecessary — Stencila loads the base theme implicitly.
- If the theme sets color tokens for light mode without corresponding `*-dark` variants, flag the dark-mode gap when the theme targets web outputs.
- If the theme claims dark-mode support for DOCX or email, flag that as incorrect.
