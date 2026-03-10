# Semantic and font token review reference

Use this file to judge whether a theme is starting from the right semantic foundation and whether font choices are portable and maintainable.

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

## Review cautions

- Do not require primitive font-stack edits unless the request actually needs them.
- Do not approve missing custom font assets or imports without noting the dependency.
- Keep exportable font and semantic choices at top-level `:root` when the user expects non-web targets to match.
- If exact token availability is uncertain, verify with `stencila themes tokens --scope semantic` instead of guessing.
