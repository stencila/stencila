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
allowed-tools: read_file glob grep snap shell
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
- [`references/snap-tool.md`](references/snap-tool.md) for visual verification with the `snap` tool and the `/_specimen` route

Also use the theme CLI as the live source of truth when available:

- `stencila themes` or `stencila themes list` to list all available themes (workspace, user, and builtin) with their type and location
- `stencila themes show [NAME]` to display the resolved CSS of a theme (omit the name for the default resolved theme); add `--verbose` to also show resolved CSS variable values — especially useful for understanding what values are inherited from the base theme
- `stencila themes tokens` to list builtin tokens, optionally filtered by `--scope` (`semantic`, `node`, `site`, `plot`, `print`) and `--family`, with `--as json|yaml|toml` for machine-readable output
- `stencila themes validate <FILE>` to check that a CSS theme parses and that custom properties correspond to known builtin design tokens; use `--strict` when unknown tokens should fail validation

## Visual verification with `snap` and `/_specimen`

Use the `snap` tool during review to verify how the theme actually renders, rather than relying solely on static analysis of the CSS. Snap returns structured measurement data (tokens, CSS properties, layout metrics, contrast ratios, color palette) by default; screenshots are opt-in when you need visual confirmation. You can snap any page served by Stencila — including the user's own documents and site pages — but the `/_specimen` route is the canonical target for theme visual QA. It is a stable, deterministic page that renders representative examples of every major content type — typography, headings, lists, blockquotes, code blocks, math, images, tables, figures, admonitions, and thematic breaks — so it exercises all token families in one page. See [`references/snap-tool.md`](references/snap-tool.md) for the full parameter reference.

### When to snap during review

- **At the start of review**: snap `/_specimen` to see how the theme currently renders, and extract token values to compare against what the CSS declares.
- **To verify specific findings**: snap with a focused `selector` or `token_prefix` to gather evidence for a specific review finding.
- **For dark-mode review**: snap with `dark: true` to verify that dark variants render correctly.
- **For responsive review**: snap with `device: "mobile"` or other presets to verify layout at different viewports.
- **For color review**: snap with `palette: true` to verify color harmony and contrast.

### Key snap parameters for theme review

| Parameter | Use for |
|---|---|
| `route: "/_specimen"` | Target the specimen page — exercises all major content types in one page |
| `measure: "theme"` | Collect measurements using the theme-oriented selector preset |
| `tokens: true` | Extract resolved CSS custom property values to verify overrides are applied. **Requires at least one `token_prefix`** — the tool returns a validation error without one |
| `token_prefix: ["text", "heading"]` | Filter token output to specific families (required with `tokens`). Use narrow prefixes — bare `"color"` matches 100+ primitive palette tokens; prefer `"color-accent"`, `"surface"`, etc. |
| `palette: true` | Extract the page's color palette for harmony and contrast review |
| `dark: true` or `light: true` | Force a color scheme to review light and dark variants |
| `device: "mobile"` | Test at a specific viewport (also `"tablet"`, `"laptop"`, `"desktop"`) |
| `full_page: true` | Capture the full scrollable page instead of just the viewport |
| `selector: "stencila-table"` | Focus on a specific element type for targeted review |
| `assert: ["css(stencila-paragraph).fontSize>=16px"]` | Assert numeric CSS properties; use `~=` for string matching (e.g., `fontFamily~=Source Serif`) |

### Typical snap workflow during review

1. Baseline snap: `snap(route: "/_specimen", measure: "theme", tokens: true, token_prefix: ["text", "heading", "surface"])` — see what the theme produces now. Measurements and tokens are returned by default without a screenshot.
2. Token verification: `snap(route: "/_specimen", tokens: true, token_prefix: ["text", "heading", "surface"])` — compare resolved values against what the CSS declares. Adjust prefixes to match the families under review.
3. Dark mode: `snap(route: "/_specimen", dark: true, tokens: true, token_prefix: ["text", "surface"])` — check for missing or broken dark variants. Add `screenshot: true` for a visual check.
4. Responsive: `snap(route: "/_specimen", device: "mobile", measure: "theme")` — check for layout or overflow issues. Run separate snaps per device preset.
5. Color harmony: `snap(route: "/_specimen", palette: true)` — verify the overall palette is cohesive.
6. Spot-check: `snap(route: "/_specimen", selector: "stencila-code-block")` — focus on a specific content type flagged in review.

Also snap the user's actual document route to verify the theme works on real content, not just the specimen.

### Screenshot tips

Screenshots are opt-in (`screenshot: true`) and default to `resize: "auto"` which preserves full resolution unless the image exceeds hard provider limits (8000px). A 600px minimum width floor prevents worst-case mobile downscaling.

- **Full-page screenshots of tall pages** (like `/_specimen`) are useful for gross layout checks but get downscaled on very tall captures, making typography details harder to read. For font rendering and spacing verification, use selector-targeted snaps (e.g., `selector: "stencila-heading"`) instead.
- **Prefer unique selectors** for element screenshots — when multiple elements match, only the first is captured but measurements cover all matches.
- **`measure: "theme"` output** has summaries and diagnostics first, followed by verbose per-element CSS. The summaries are typically the most useful part — scan those before diving into raw values.
- **Diagnostics about unmatched selectors** (e.g., "Selector footer matched no elements") are informational facts about the route, not theme bugs. Do not turn them into review findings unless the route is actually expected to contain those elements.
- **Contrast calculations** work on elements with explicit backgrounds. Transparent descendants report "background color is unavailable" — this is expected, not a theme defect. Inspect enclosing solid-surface containers when contrast data is needed.

Theme resolution is cached for about 30 seconds. When reviewing iterative changes, batch CSS edits before snapping rather than snapping after each small modification.

### When snap is unavailable

If `snap` cannot be run — for example, no Stencila server is running, the `/_specimen` route is not available, or the theme is not yet renderable — do not fabricate visual findings. Instead:

- State that visual verification is **pending** and explain why (e.g., "no running Stencila instance" or "theme not yet applied to a served route").
- Fall back to static CSS analysis, token verification via the CLI (`stencila themes validate`, `stencila themes tokens`), and the review-lens dimensions that do not require rendering.
- Include a concrete recommendation for which `snap` commands to run once the environment is available.
- Never claim snap-based findings unless `snap` was actually executed and returned results.

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
6. Visual rendering
   - Does a `snap` of `/_specimen` confirm the theme renders as intended across all content types?
   - Do dark-mode and responsive snaps reveal issues not visible in the CSS alone?
   - Does the color palette extracted by `snap` look cohesive?
7. Validation and approval readiness
   - Should `stencila themes validate <FILE>` or `--strict` be run?
   - Should exact names be checked with `stencila themes tokens --scope ... --family ...`?
   - Has `snap` been used to visually verify the rendered result on `/_specimen`?
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
   - Use `snap` to capture a visual baseline of how the theme currently renders:
     - `snap(route: "/_specimen", measure: "theme", tokens: true, token_prefix: ["text", "heading", "surface"])` — see current measurements and resolved token values before reviewing proposed changes. Adjust prefixes to match the families relevant to the review.
     - This provides concrete evidence for review findings rather than relying on mental CSS evaluation.
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
   - Use `snap(route: "/_specimen", dark: true, tokens: true, token_prefix: ["text", "surface"])` to verify dark-mode token values. Add `screenshot: true` for a visual check — this often reveals issues that are not obvious from reading the CSS alone.
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
     7. visual verification (snap results from `/_specimen` and user content, or "pending" if snap was not run)
     8. validation and verification commands
     9. final verdict or recommended next step

11. Keep corrections secondary and explicitly requested.
    - Primary mode is assess-and-report.
    - Only after completing the review, and only if the user explicitly asks, provide a minimal corrective example.
    - Keep any example tightly scoped to the findings you already reported.

12. Recommend validation with target-specific checks.
    - When a concrete theme file path exists, recommend or run `stencila themes validate <FILE>` before concluding.
    - Use `stencila themes validate <FILE> --strict` when unknown tokens should fail review.
    - Use `stencila themes validate <FILE> --as json` when machine-readable validation results are useful.
    - Use `snap` to provide visual evidence for your review findings:
      - `snap(route: "/_specimen", measure: "theme", tokens: true, token_prefix: ["text", "heading", "surface"])` — verify content types render correctly and confirm resolved token values. Adjust prefixes to match the families under review.
      - `snap(route: "/_specimen", dark: true, tokens: true, token_prefix: ["text", "surface"])` — verify dark-mode token values.
      - `snap(route: "/_specimen", device: "mobile", measure: "theme")` — verify responsive behavior. Run separate snaps per device.
      - `snap(route: "/_specimen", palette: true)` — verify color harmony.
      - Also snap the user's actual document or site route to confirm real-content rendering.
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
7. Visual verification (snap results from `/_specimen` and user content)
8. Validation and token-verification commands
9. Final verdict or next step

Each finding should be evidence-based. Use `snap` measurements, tokens, and screenshots as evidence alongside CSS analysis — do not make speculative claims without citing the token, selector, file region, snippet, snap result, or target assumption involved.

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
7. Visual verification:
   - `snap(route: "/_specimen", measure: "theme", tokens: true, token_prefix: ["text", "heading", "surface", "border"])` confirmed typography tokens applied correctly; table borders render with the selector-level rule but would be more portable as token overrides.
   - `snap(route: "/_specimen", dark: true, tokens: true, token_prefix: ["text", "surface"])` revealed no dark variants defined — text and surface colors fall back to base-theme defaults in dark mode, which may not match the intended design.
8. Validation and token-verification commands:
   - `stencila themes validate theme.css --strict`
   - `stencila themes tokens --scope print`
   - `stencila themes tokens --scope node --family table`
9. Final verdict or next step:
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
7. Visual verification:
   - `snap(route: "/_specimen", device: "mobile", measure: "theme")` confirmed the 48px header causes nav items to wrap on mobile — consider increasing the height or switching to a hamburger menu at narrow viewports.
   - `snap(route: "/_specimen", measure: "theme")` showed the letter-spacing override applies correctly but is not achievable through tokens alone, justifying the selector in this case.
8. Validation and token-verification commands:
   - `stencila themes tokens --scope site --family nav-menu`
   - `stencila themes tokens --scope site --family site-search`
   - `stencila themes validate theme.css`
9. Final verdict or next step:
   - likely acceptable after exact-name verification and a responsive header fix for narrow viewports

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
- If `snap` cannot be run (no server, no served route, theme not yet renderable), mark visual verification as "pending" in the review output, rely on static CSS and CLI analysis, and recommend specific `snap` commands to run once the environment is ready. Do not claim snap-based findings without actual snap results.
