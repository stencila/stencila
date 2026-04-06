---
title: "Theme Creation Skill"
description: "Create, update, or plan a Stencila theme for documents or published sites. Use when asked to choose a theme direction, write or patch theme.css, recommend semantic or module token families, customize site navigation or branding, tune PDF and print page tokens, align web, Python, and R plots with a Stencila design system, list available builtin tokens with `stencila themes tokens`, or validate a theme file with `stencila themes validate`."
keywords:
  - theme.css
  - document theme
  - site theme
  - design system
  - theme tokens
  - semantic tokens
  - plot tokens
  - print tokens
  - css custom properties
  - pdf theme
  - docx theme
  - email theme
  - site branding
  - navigation theming
  - logo styling
  - dark mode
  - font-face
  - self-hosted fonts
---

Create, update, or plan a Stencila theme for documents or published sites. Use when asked to choose a theme direction, write or patch theme.css, recommend semantic or module token families, customize site navigation or branding, tune PDF and print page tokens, align web, Python, and R plots with a Stencila design system, list available builtin tokens with `stencila themes tokens`, or validate a theme file with `stencila themes validate`.

**Keywords:** theme.css · document theme · site theme · design system · theme tokens · semantic tokens · plot tokens · print tokens · css custom properties · pdf theme · docx theme · email theme · site branding · navigation theming · logo styling · dark mode · font-face · self-hosted fonts

> [!tip] Usage
>
> To use this skill, add `theme-creation` to the `allowed-skills` list in your agent's AGENT.md. You can also ask `#agent-creator` to build an agent that uses it.

# Configuration

| Property | Value |
| -------- | ----- |
| Allowed tools | `read_file`, `write_file`, `edit_file`, `apply_patch`, `glob`, `grep`, `shell`, `snap`, `ask_user` |

# Instructions

## Overview

Help the user create, update, or plan a Stencila theme. Depending on the request, this may mean producing a concrete artifact such as a workspace `theme.css`, a patch to an existing theme file, or a reusable CSS snippet, or it may mean producing an advisory plan such as token recommendations, target tradeoffs, branding direction, and a validation checklist before any file is edited.

Use the localized references in this skill directory instead of `site/docs/themes/`:

- [`references/themes-guide.md`](references/themes-guide.md) for architectural context and workflow background when you need it; the numbered steps below remain the primary operating procedure
- [`references/semantic-and-font-tokens.md`](references/semantic-and-font-tokens.md) for semantic and font tokens
- [`references/node-token-families.md`](references/node-token-families.md) for document/node token families and guidance on when to use them
- [`references/print-and-pdf-tokens.md`](references/print-and-pdf-tokens.md) for print and PDF page tokens
- [`references/plot-tokens.md`](references/plot-tokens.md) for plot theming tokens
- [`references/site-token-families.md`](references/site-token-families.md) for site-theme component families and verified exact site token names
- [`references/cli-commands.md`](references/cli-commands.md) for token discovery and validation commands
- [`references/snap-tool.md`](references/snap-tool.md) for visual verification with the `snap` tool and the `/_specimen` route

Also use the theme CLI as a live source of truth when available:

- `stencila themes` or `stencila themes list` to list all available themes (workspace, user, and builtin) with their type and location
- `stencila themes show [NAME]` to display the resolved CSS of a theme (omit the name for the default resolved theme); add `--verbose` to also show resolved CSS variable values
- `stencila themes new [NAME]` to scaffold a new theme — omit the name for a workspace `theme.css`, or provide a name for a user theme; use `--force` to overwrite
- `stencila themes tokens` to list builtin tokens, optionally filtered by `--scope` (`semantic`, `node`, `site`, `plot`, `print`) and `--family`, with `--as json|yaml|toml` for machine-readable output
- `stencila themes validate <FILE>` to check that a CSS theme parses and that custom properties correspond to known builtin design tokens; use `--strict` when unknown tokens should fail validation
- `stencila themes remove <NAME>` to remove a user theme; use `--force` to skip the confirmation prompt

## Visual verification with `snap` and `/_specimen`

Use the `snap` tool throughout theme creation to verify how changes render. Snap returns structured measurement data (tokens, CSS properties, layout metrics, contrast ratios, color palette) by default; screenshots are opt-in when you need visual confirmation. You can snap any page served by Stencila — including the user's own documents and site pages — but the `/_specimen` route is the canonical target for theme visual QA. It is a stable, deterministic page that renders representative examples of every major content type — typography, headings, lists, blockquotes, code blocks, math, images, tables, figures, admonitions, and thematic breaks — so it exercises all token families in one page. See [`references/snap-tool.md`](references/snap-tool.md) for the full parameter reference.

### When to snap

- **Before starting**: snap `/_specimen` to see the current baseline appearance and token values before making any changes.
- **After each significant change**: snap again to confirm the change looks correct and has no unintended side-effects on other content types.
- **At the end**: snap the final result at different viewports and color schemes for a comprehensive check.

### Key snap parameters for theme work

| Parameter | Use for |
|---|---|
| `route: "/_specimen"` | Target the specimen page — exercises all major content types in one page |
| `measure: "theme"` | Collect measurements using the theme-oriented selector preset |
| `tokens: true` | Extract resolved CSS custom property values to verify token overrides took effect. **Requires at least one `token_prefix`** — the tool returns a validation error without one |
| `token_prefix: ["text", "heading"]` | Filter token output to specific families (required with `tokens`). Use narrow prefixes — bare `"color"` matches 100+ primitive palette tokens; prefer `"color-accent"`, `"surface"`, etc. |
| `palette: true` | Extract the page's color palette to check color harmony |
| `dark: true` or `light: true` | Force a color scheme to verify light and dark variants |
| `device: "mobile"` | Test responsive behavior at a specific viewport (also `"tablet"`, `"laptop"`, `"desktop"`) |
| `full_page: true` | Capture the full scrollable page instead of just the viewport |
| `selector: "stencila-table"` | Focus on a specific element type |
| `assert: ["css(stencila-paragraph).fontSize>=16px"]` | Assert numeric CSS properties; use `~=` for string matching (e.g., `fontFamily~=Source Serif`) |

### Typical snap workflow during theme creation

1. Baseline snap: `snap(route: "/_specimen", measure: "theme", tokens: true, token_prefix: ["text", "heading", "surface"])` — see what you are starting from. Measurements and tokens are returned by default without a screenshot.
2. After setting semantic tokens: `snap(route: "/_specimen", tokens: true, token_prefix: ["text", "heading", "surface"])` — verify the token overrides took effect.
3. After adding module tokens: `snap(route: "/_specimen", selector: "stencila-code-block", tokens: true, token_prefix: ["code"])` — spot-check specific node types and verify their tokens.
4. Dark mode check: `snap(route: "/_specimen", dark: true, tokens: true, token_prefix: ["text", "surface"])` — verify dark-mode token values. Add `screenshot: true` for a visual check.
5. Responsive check: `snap(route: "/_specimen", device: "mobile", measure: "theme")` — verify layout at a specific viewport. Run separate snaps for each device preset.
6. Color harmony: `snap(route: "/_specimen", palette: true)` — review the overall color palette.

You can also snap the user's actual document route (e.g., `route: "/"` or `route: "/docs/"`) alongside `/_specimen` to verify the theme looks correct on real content too. When choosing a route from source files, remember that `index.*`, `main.*`, and `README.*` act as the `index.html` for their containing directory, so `docs/README.md`, `docs/main.md`, and `docs/index.md` all render at `"/docs/"`.

### Screenshot tips

Screenshots are opt-in (`screenshot: true`) and default to `resize: "auto"` which preserves full resolution unless the image exceeds hard provider limits (8000px). A 600px minimum width floor prevents worst-case mobile downscaling.

- **Full-page screenshots of tall pages** (like `/_specimen`) are useful for gross layout checks but get downscaled on very tall captures, making typography details harder to read. For font rendering and spacing verification, use selector-targeted snaps (e.g., `selector: "stencila-heading"`) instead.
- **Prefer unique selectors** for element screenshots — when multiple elements match, only the first is captured but measurements cover all matches.
- **`measure: "theme"` output** has summaries and diagnostics first, followed by verbose per-element CSS. The summaries are typically the most useful part — scan those before diving into raw values.
- **Diagnostics about unmatched selectors** (e.g., "Selector footer matched no elements") are informational facts about the route, not theme bugs. Do not turn them into review findings unless the route is actually expected to contain those elements.

Theme resolution is cached for about 30 seconds. Batch your CSS changes before snapping rather than snapping after each small edit.

### When snap is unavailable

If `snap` cannot be run — for example, no Stencila server is running, the `/_specimen` route is not available, or the theme is not yet renderable — do not fabricate visual findings. Instead:

- State that visual verification is **pending** and explain why (e.g., "no running Stencila instance" or "theme not yet applied to a served route").
- Continue with CLI validation (`stencila themes validate`) and token verification (`stencila themes tokens`) as the primary quality checks.
- Include a concrete recommendation for which `snap` commands to run once the environment is available.
- Never claim snap-based findings unless `snap` was actually executed and returned results.

Stencila themes are token-first. Prefer semantic tokens as the stable public API, use module-specific tokens only when needed, and add custom CSS rules only where tokens are insufficient.

## Core rules

- Prefer semantic tokens first.
- Keep exportable cross-target tokens at top-level `:root`.
- Use `--plot-*` tokens for plot targets.
- Do not assume parity across web, PDF, DOCX, email, Python, and R.
- Inspect existing theme files before patching or replacing them.
- Prefer `stencila themes tokens` to confirm builtin token availability and scope/family coverage when exact names matter or when the localized references may have drifted.
- Use the localized references to explain how token families are applied, cross-target constraints, and theme architecture; use the CLI for comprehensive current inventories.
- If a needed exact name is not localized here and you cannot verify it with `stencila themes tokens`, describe the token family and intended effect instead of guessing.
- Do not invent missing asset files; ask for them or use a clearly marked placeholder path only when the user needs a concrete implementation.

## Dark mode

Many tokens have `*-dark` variants (e.g., `--text-color-primary-dark`, `--surface-background-dark`, `--plot-background-dark`). The base theme applies these automatically via `prefers-color-scheme: dark`. When creating a theme:

- Override the light-mode token for both light and dark if the value is suitable for both schemes.
- Override the `*-dark` variant explicitly when light and dark values need to differ (e.g., `--color-accent` and `--color-accent-dark`).
- Check dark variants for plot, surface, and text tokens especially — colors that work on a light background often need adjustment for dark backgrounds.
- Use `stencila themes tokens --scope semantic` or `--scope plot` to see which tokens have dark variants.
- Dark variants are only relevant for web and HTML-derived outputs; non-web targets such as DOCX and email do not use dark mode.

## Starting template

Stencila always loads `base.css` implicitly before any theme CSS — both when rendering HTML (as a separate `<link>` tag) and when computing theme variables (base variables are merged before theme-specific ones). Users do not have `base.css` in their workspace, so **do not include `@import url("./base.css")`** in theme files. A workspace `theme.css` or user theme should contain only `:root` overrides and any additional rules or font imports.

```css
:root {
  --text-font-family: "Source Serif 4", Georgia, serif;
  --heading-font-family: "Inter", Arial, sans-serif;
  --text-color-primary: #1f2937;
  --color-accent: #0f766e;
  --surface-background: #ffffff;
  --content-width: 72ch;
  --content-spacing: 1.25rem;
  --border-color-default: #d1d5db;
}
```

If the theme needs custom web fonts, add `@import url(...)` for the font provider (e.g., Google Fonts) before the `:root` block. For self-hosted fonts, use `@font-face` rules before `:root` instead:

```css
@font-face {
  font-family: "Custom Font";
  src: url("./fonts/custom-font.woff2") format("woff2");
  font-weight: normal;
  font-display: swap;
}

:root {
  --font-family-serif: "Custom Font", var(--font-family-serif);
  --text-font-family: var(--font-family-serif);
}
```

Then add only the module-specific tokens or focused selectors needed for the user’s goal.

## Steps

1. Choose the workflow branch early.
   - Decide whether the user wants **plan only** or **implement now**.
   - If the request is strategic, exploratory, or direction-setting, use the plan-only branch.
   - If the request asks for CSS, file edits, patches, or ready-to-paste snippets, use the implement-now branch.
   - If one missing detail changes the artifact materially, ask a clarifying question; otherwise proceed with a clearly stated assumption.

   | Situation | Default action |
   |---|---|
   | User asks for direction, options, or tradeoffs | Assume **plan only** |
   | User asks for CSS, a patch, or file edits | Assume **implement now** |
   | Document theme vs site theme is unclear and published-site chrome may be affected | Ask a clarifying question |
   | Request is clearly article/report typography focused with no target stated | Assume **document theme** and state the assumption |
   | Existing file likely exists and patch-vs-rewrite changes the result materially | Inspect first or ask if inspection is not possible |

2. Identify the theme type and targets.
   - Determine whether they need a **document theme**, **site theme**, **plot theme**, **print/PDF theme**, or a theme covering several outputs.
   - Ignore workflow-process phrasing unless it changes actual theme requirements.
   - If the user does not specify document vs site, infer **document theme** only when the request is clearly about article/report typography or content nodes.
   - Ask a clarifying question when the request could plausibly affect published-site chrome, navigation, branding, or layout.

3. Inspect the existing theme context when implementation is requested.
   - If files are available, look for an existing `theme.css` and related assets before generating new CSS.
   - Use `stencila themes show` to see the current default resolved theme CSS, or `stencila themes show <NAME>` for a specific theme. Add `--verbose` to also see resolved variable values — this is especially useful for understanding what values are inherited from the base theme.
   - Use `stencila themes list` to see all available themes (workspace, user, and builtin) and their locations.
   - Reuse the current token vocabulary, selectors, and import style when updating a theme.
   - Patch requested selectors or token blocks in place when possible instead of rewriting the full file.
   - If fonts, logos, icons, or other assets are referenced, use existing workspace asset paths when available.
   - Keep Stencila resolution order in mind when explaining placement: when no theme is specified, Stencila resolves workspace `theme.css` first while walking up from the document path, then user `default.css`, then base theme (no overrides).
   - If the user refers to a named theme, remember that named themes are resolved from user themes first, then builtin themes.
   - Use `stencila themes new` to scaffold a workspace `theme.css` template, or `stencila themes new <NAME>` for a named user theme, when starting from scratch.

4. Start from the semantic foundation.
   - Use [`references/semantic-and-font-tokens.md`](references/semantic-and-font-tokens.md) for semantic starting points, stable font patterns, and guidance on how semantic tokens are applied.
   - Prefer semantic tokens first:
     - typography tokens such as `--text-font-family`, `--heading-font-family`, `--code-font-family`
     - color and surface tokens such as `--text-color-primary`, `--color-accent`, `--surface-background`
     - layout and framing tokens such as `--content-width`, `--content-spacing`, `--border-color-default`
   - Use primitive font-stack tokens only when you need to add or extend fallback stacks.

5. Verify exact module-specific tokens before using them.
   - Do not rely only on summarized guides when concrete token names matter.
   - Use the CLI first for comprehensive and current inventories:
     - `stencila themes tokens`
     - `stencila themes tokens --scope semantic`
     - `stencila themes tokens --scope site --family nav-menu`
     - `stencila themes tokens --scope plot --as json`
   - Then inspect the focused localized reference files for workflow and application guidance:
     - semantic and fonts: [`references/semantic-and-font-tokens.md`](references/semantic-and-font-tokens.md)
     - node/document families: [`references/node-token-families.md`](references/node-token-families.md)
     - print and PDF: [`references/print-and-pdf-tokens.md`](references/print-and-pdf-tokens.md)
     - plots: [`references/plot-tokens.md`](references/plot-tokens.md)
     - site families: [`references/site-token-families.md`](references/site-token-families.md)
   - Only emit exact module-specific token names that are verified by the CLI, the localized references, or both.

6. Branch by theme type.
   - **Document theme**: usually start with semantic tokens, then inspect [`references/node-token-families.md`](references/node-token-families.md) and add node/module token families for headings, paragraphs, lists, links, code, tables, figures, citations, references, plots, and print/page behavior as needed.
   - **Site theme**: keep the document foundation, then inspect `references/site-token-families.md` for exact names in families such as `layout`, `nav-menu`, `nav-tree`, `nav-groups`, `breadcrumbs`, `toc-tree`, `prev-next`, `logo`, `title`, and `site-search`.
   - Use site terminology consistently:
     - component family names include `title` and `site-search`
     - exact site-title tokens are prefixed `--site-title-*`
     - exact site-search tokens are prefixed `--search-*`
     - breadcrumbs are treated as a site navigation surface even though the localized source listing currently comes from a node-token path
   - **Plot theme**: inspect `references/plot-tokens.md` and use explicit `--plot-*` tokens.
   - **Print/PDF theme**: inspect `references/print-and-pdf-tokens.md` and keep exportable page tokens in top-level `:root`.

7. Keep cross-target values exportable.
   - Put tokens that should affect DOCX, email, PDF, and plots at top-level `:root`.
   - Do not rely on tokens inside `@media` or `@supports` for non-web targets.
   - You may still use responsive rules or `@media print` for web-only or print-only presentation, but state clearly that these are not exported as general non-web theme tokens.
   - `var()`, `calc()`, and `color-mix()` can still be useful because they are resolved before target mapping.

8. Add CSS rules only where tokens are insufficient.
   - Prefer token overrides over broad selector overrides.
   - Add focused selectors for behaviors that cannot be expressed cleanly through tokens alone.
   - Keep custom selectors compact and maintainable.
   - Do not imply that arbitrary browser CSS will translate unchanged to all non-web outputs.

9. Produce the right kind of output.
   - **Plan only**:
     - focus on theme type, output targets, semantic foundation, module token families, asset needs, risks, and validation plan
     - do not default to full CSS unless the user asks for it
     - provide staged implementation guidance when useful
   - **Implement now**:
     - inspect existing files first when possible
     - produce one of these:
       - full `theme.css` content
       - a patch to an existing theme file
       - a small insertion snippet with placement guidance
     - default to `:root` overrides (do not include `@import url("./base.css")` since Stencila loads the base theme implicitly)

10. Validate with target-specific checks.
    - When implementation is requested and you have a concrete theme file path, recommend or run `stencila themes validate <FILE>` before finishing.
    - Use `stencila themes validate <FILE> --strict` when the user wants unknown tokens treated as errors.
    - Use `snap` to visually verify the rendered result on the specimen page and the user's actual content:
      - `snap(route: "/_specimen", measure: "theme", tokens: true, token_prefix: ["text", "heading", "surface"])` — verify content types render correctly and key token overrides took effect. Adjust the prefix list to match the token families you changed.
      - `snap(route: "/_specimen", dark: true, tokens: true, token_prefix: ["text", "surface"])` — verify dark-mode token values if dark variants were set. Add `screenshot: true` for a visual check.
      - `snap(route: "/_specimen", device: "mobile", measure: "theme")` — verify responsive behavior at mobile viewport. Run separate snaps per device.
      - `snap(route: "/_specimen", palette: true)` — verify color harmony.
      - Also snap the user's actual document or site route to confirm the theme works on real content. Prefer the rendered directory route when the source file is `index.*`, `main.*`, or `README.*`.
    - State plainly that Stencila translates themes across targets, but not every web CSS rule or module maps unchanged to PDF, DOCX, email, Python, or R.
    - Recommend concrete checks for each required target:
      - HTML or site preview for screen behavior, responsive layout, and site chrome styling
      - PDF or print preview for page size, margins, header/footer content, first-page overrides, and page-fit of tables/figures/code blocks
      - DOCX for heading, body, link, table, figure-caption, and page-layout mapping
      - email output for simplified typography, spacing, surfaces, and safe component styling
      - Python/R execution for `--plot-*` token transfer to plots
    - If quick iteration matters, mention that runtime theme resolution is cached for about 30 seconds and watch-based tooling is needed for immediate refresh.

## Output guidelines

When useful, structure the response like this:

1. Workflow branch: plan only or implement now
2. Theme type and target(s)
3. File choice (`theme.css`, patch, snippet, or plan)
4. Semantic token foundation
5. Verified module token families or exact tokens used
6. Focused CSS selectors, if any
7. Target-specific caveats
8. Validation checklist
9. Final artifact or advisory output

See [`references/cli-commands.md`](references/cli-commands.md) for CLI commands and [`references/snap-tool.md`](references/snap-tool.md) for visual verification.

## Examples

Input: We have not started coding yet. Help us choose a theme direction for a documentation site that should feel calm, technical, and accessible across web and PDF.

Output:

1. Workflow branch: plan only
2. Theme type and target(s): site theme with PDF-aware document tokens for web and PDF
3. File choice: plan only; do not emit CSS yet because branding assets and site scope are still being confirmed
4. Semantic token foundation:
   - `--text-font-family`, `--heading-font-family`, `--text-color-primary`, `--surface-background`, `--color-accent`, `--border-color-default`
   - `--content-width`, `--content-spacing`
5. Verified module token families or exact tokens used:
   - inspect `layout`, `nav-menu`, `nav-tree`, `breadcrumbs`, `toc-tree`, `title`, `logo`, and `site-search`
   - add top-level page tokens from `references/print-and-pdf-tokens.md`
6. Focused CSS selectors, if any: none yet; start with tokens first
7. Target-specific caveats:
   - keep essential branding in top-level `:root` tokens rather than responsive-only rules
   - expect site chrome and interaction details to remain more web-specific than PDF output
8. Validation checklist:
   1. preview the site chrome and reading experience in HTML
   2. render PDF and compare margins, headers/footers, and content width
   3. verify focus states and contrast in navigation and search flows
   4. confirm logo assets and any custom fonts before implementation
9. Final artifact or advisory output:
   - recommended direction: calm serif-or-neutral body text, restrained accent color, quiet navigation surfaces, and moderate content width
   - next implementation step: confirm whether the request includes custom logo/search styling or only typography and navigation

Input: Create a document theme for research reports with serif body text, sans-serif headings, slightly wider reading width, subtle table borders, and PDF-friendly page margins.

Output:

```css
:root {
  --text-font-family: "Source Serif 4", Georgia, serif;
  --heading-font-family: "Inter", Arial, sans-serif;
  --text-color-primary: #1f2937;
  --surface-background: #ffffff;
  --content-width: 74ch;
  --content-spacing: 1.25rem;
  --border-color-default: #d6dae1;

  --page-width: 210mm;
  --page-height: 297mm;
  --page-margin-top: 18mm;
  --page-margin-right: 16mm;
  --page-margin-bottom: 20mm;
  --page-margin-left: 16mm;
}

stencila-table {
  --table-border-color: var(--border-color-default);
}
```

Notes:
- Start with verified semantic tokens, then add verified page tokens.
- The exact page token names shown here are verified example tokens; when working live, re-check with `stencila themes tokens --scope print` if exact names matter.
- Keep the page tokens at top-level `:root` so they can be exported to print/PDF flows.
- This is a document theme, so use only a focused table override instead of broad selector rewrites.

Input: Create a minimal document theme with teal accents and verify it renders correctly.

Output:

```css
:root {
  --text-font-family: "Inter", Arial, sans-serif;
  --text-color-primary: #1f2937;
  --color-accent: #0f766e;
  --surface-background: #fafafa;
  --content-width: 70ch;
  --border-color-default: #d1d5db;
}
```

Validation checklist:
1. `stencila themes validate theme.css` — confirm tokens parse and are recognized.
2. `snap(route: "/_specimen", tokens: true, token_prefix: ["text", "color-accent", "surface"])` — verify the semantic foundation and token overrides took effect.
3. `snap(route: "/_specimen", dark: true, tokens: true, token_prefix: ["text", "surface"])` — check that the base-theme dark fallbacks are acceptable (no explicit dark variants were set, so base defaults apply). Add `screenshot: true` for a visual check.
4. `snap(route: "/_specimen", device: "mobile", measure: "theme")` — confirm mobile layout.
5. `snap(route: "/_specimen", palette: true)` — verify the teal accent integrates with surrounding colors.

If `snap` cannot be run (e.g., no Stencila server is running), mark visual verification as **pending** and rely on `stencila themes validate` and `stencila themes tokens --scope semantic` for now. Recommend running the snap commands above once the server is available.

Input: Update our published site theme so the header is slimmer, navigation is quieter, the site title is more prominent, and search feels more branded.

Output:

```css
:root {
  --layout-header-height: 56px;
  --header-background: #ffffff;
  --header-border-color: #e5e7eb;

  --nav-menu-color: #4b5563;
  --nav-menu-color-hover: #111827;
  --nav-menu-color-active: #0f766e;
  --nav-menu-item-padding-x: var(--space-2);

  --site-title-font-size: var(--font-size-xl);
  --site-title-font-weight: 700;
  --site-title-color: #111827;

  --search-modal-width: 34rem;
  --search-highlight-color: #0f766e;
}
```

Notes:
- These exact names are verified example tokens from `references/site-token-families.md`.
- In live execution, also confirm them with `stencila themes tokens --scope site --family nav-menu` and related family queries when exact names matter.
- This is a site theme, so include site token families rather than only document-body tokens.
- Add `logo`, `breadcrumbs`, `toc-tree`, or `prev-next` tokens only if the user asks for those surfaces.

Input: Make Python and R plots match the document branding with teal accents, softer grid lines, and consistent typography.

Output:

```css
:root {
  --plot-theme: custom;
  --plot-font-family: "Inter", Arial, sans-serif;
  --plot-font-size: 11pt;
  --plot-text-color: #1f2937;
  --plot-grid-color: #d1d5db;
  --plot-axis-line-color: #6b7280;
  --plot-title-size: 14pt;
  --plot-color-1: #0f766e;
  --plot-color-2: #0ea5a4;
  --plot-color-3: #334155;
}
```

Notes:
- Only `--plot-*` tokens transfer to Python and R kernels.
- The exact names shown here are verified example plot tokens; when working live, re-check with `stencila themes tokens --scope plot` before expanding the set.
- Check exact plot tokens with `stencila themes tokens --scope plot` and use `references/plot-tokens.md` to understand cross-target caveats before adding narrower plot variables.
- Describe parity as approximate unless the user has validated each renderer.

Input: Help us theme this, but we have not decided whether it is for the published site or for the article pages themselves.

Output:

Before proposing tokens or CSS, ask a clarifying question such as:

> Do you want a document theme for the article/report content itself, a published-site theme for navigation/branding/chrome, or both?

Then proceed once the answer makes the target surfaces clear.

Input: Update the existing `theme.css` so links use our accent color and tables keep their current styling.

Output:

```diff
--- a/theme.css
+++ b/theme.css
@@
 :root {
   --text-font-family: "Source Serif 4", Georgia, serif;
-  --color-accent: #2563eb;
+  --color-accent: #0f766e;
+  --link-color: var(--color-accent);
+  --link-color-hover: #115e59;
 }
```

Notes:
- Inspect the existing file first and preserve its imports, token naming, and unrelated selectors.
- Patch only the requested link token block or selector area instead of regenerating the whole theme.
- Because the user asked to keep table styling, leave existing table selectors and table tokens untouched.

Input: Patch our existing `theme.css` after inspection so it keeps the current font imports and table rules, but updates links and accent color for a calmer teal brand.

Output:

1. Inspect `theme.css` first.
2. Keep any existing `@import url(...)` font lines unchanged.
3. Update only the relevant `:root` link and accent tokens.
4. Leave unrelated selectors such as `stencila-table` rules untouched.

```diff
--- a/theme.css
+++ b/theme.css
@@
 @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
 
 :root {
   --text-font-family: "Inter", Arial, sans-serif;
-  --color-accent: #2563eb;
+  --color-accent: #0f766e;
+  --link-color: var(--color-accent);
+  --link-color-hover: #115e59;
 }
```

Notes:
- Preserve existing imports and any unrelated token blocks or selectors.
- Do not rewrite the whole file when the request only changes a small token set.
- If the file already has explicit link tokens, patch those in place instead of adding duplicates elsewhere.

## Edge Cases

- If the user gives no output target and the request is implementation-oriented, default to a simple document-oriented theme only when the request is clearly document-centric, and state that assumption.
- If the user gives no output target and the request is mainly strategic or branding-oriented, ask which outputs matter before committing to file structure.
- If the user does not specify whether the theme is for documents or published sites, infer document theme only for clearly document-centric requests; otherwise ask a clarifying question.
- If the user does not say whether an existing theme file already exists, inspect the workspace when possible. If inspection is not possible and patch-vs-new-file changes the answer materially, ask.
- If the user does not specify which output targets matter, ask when parity constraints are important; otherwise default to web-first document guidance and explicitly note that non-web mappings may differ.
- If the user mentions fonts or assets without confirming availability, do not invent files. Ask for asset paths when they are required for a concrete implementation; for planning requests, recommend token families and fallback stacks first.
- If the user asks for a site theme, expand beyond document tokens into verified site token families.
- If the user asks for a specific module-specific token that is not localized in this skill’s references, do not guess; describe the family and tell the user the exact name should be verified before implementation.
- If the user depends on DOCX or email parity, explicitly call out unsupported, not-yet-mapped, or simplified areas instead of claiming full fidelity.
- If important exported tokens are placed inside `@media` or `@supports`, move them to top-level `:root` and explain why.
- If the user wants plot styling in Python or R, ensure `--plot-*` tokens are present even if the rest of the document theme is already defined.
- If the theme references assets that are not present in the workspace, do not fabricate files; keep a clear placeholder path or request the missing assets.
- If exact token availability is uncertain, run `stencila themes tokens` with the narrowest useful `--scope` and `--family` filters instead of guessing from memory.
- If a concrete theme file is being created or updated, validate it with `stencila themes validate <FILE>` and mention `--strict` when unknown-token failures are desirable.
- If `snap` cannot be run (no server, no served route, theme not yet renderable), mark visual verification as "pending" in the output, rely on CLI validation and token verification, and recommend specific `snap` commands to run once the environment is ready. Do not claim snap-based findings without actual snap results.

## Maintainer guidance

- Keep the localized references in `references/` focused on workflow guidance, architecture, application patterns, and caveats that the CLI token listing does not provide.
- Reduce or avoid large static token inventories when `stencila themes tokens` can provide a more current comprehensive list, to minimize drift.

---

This page was generated from [`.stencila/skills/theme-creation/SKILL.md`](https://github.com/stencila/stencila/blob/main/.stencila/skills/theme-creation/SKILL.md).
