# Semantic and font token review reference

Use this file to judge whether a theme is starting from the right semantic foundation and whether font choices are portable and maintainable.

For a comprehensive current list of builtin semantic tokens, use the CLI:

```sh
stencila themes tokens --scope semantic
```

Use machine-readable output when you need to inspect or post-process the inventory:

```sh
stencila themes tokens --scope semantic --as json
```

## Review principle

Prefer these top-level `:root` semantic tokens first:

- `--text-font-family`
- `--heading-font-family`
- `--code-font-family`
- `--math-font-family`
- `--text-color-primary`
- `--color-accent`
- `--surface-background`
- `--content-spacing`
- `--content-width`
- `--border-radius-default`
- `--border-color-default`

If typography, color, surface, spacing, or layout goals are being solved with broad selectors before these semantic tokens are considered, flag that as a maintainability and portability risk.

## What good usage looks like

- semantic tokens are defined at top-level `:root`
- fallback stacks are preserved or extended rather than replaced with a single font name
- semantic tokens carry the main design decisions, with module tokens used only for narrower exceptions

## Font-stack guidance

Primitive font stack tokens are useful when a theme needs to extend default fallbacks:

- `--font-family-serif`
- `--font-family-sans`
- `--font-family-mono`
- `--font-family-math`

A common pattern is to extend those primitive stacks and then route them through semantic tokens.

Example of correct usage:

```css
:root {
  --font-family-serif: "Source Serif 4", var(--font-family-serif);
  --font-family-sans: "Inter", var(--font-family-sans);
  --text-font-family: var(--font-family-serif);
  --heading-font-family: var(--font-family-sans);
}
```

## How to use the CLI and this reference together

1. Use `stencila themes tokens --scope semantic` to see what semantic tokens currently exist.
2. Check whether the theme starts from the smallest semantic foundation that covers the user's typography, color, surface, spacing, and layout goals.
3. Use this reference for review patterns that the token list does not show, such as fallback-stack extension and the preference for semantic tokens as the stable public API.
4. Flag module-specific tokens that duplicate semantic-level concerns.

## Dark-mode variant review

Many semantic tokens have `*-dark` variants applied via `prefers-color-scheme: dark`. Key dark-variant tokens include:

- `--text-color-primary-dark`
- `--color-accent-dark`
- `--surface-background-dark`
- `--surface-elevated-dark`

When reviewing, check whether the theme sets light-mode color tokens without corresponding `*-dark` variants when the values would not work on a dark background. Use `stencila themes tokens --scope semantic` to see which tokens have dark variants.

## Review cautions

- Do not require primitive font-stack edits unless the request actually needs them.
- Do not approve missing custom font assets or imports without noting the dependency.
- Keep exportable font and semantic choices at top-level `:root` when the user expects non-web targets to match.
- If exact token availability is uncertain, verify with `stencila themes tokens --scope semantic` instead of guessing.
- Flag `@font-face` or `@import` font declarations that appear after `:root` blocks instead of before them.
- Flag single-font-name values without fallback stacks as a fragility risk.
