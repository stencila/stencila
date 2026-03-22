# Semantic and font token reference

Use this file for semantic starting points, font-stack patterns, and guidance on how semantic tokens are applied in a Stencila theme.

For a comprehensive current list of builtin semantic tokens, use the CLI:

```sh
stencila themes tokens --scope semantic
```

Use machine-readable output when you need to inspect or post-process the inventory:

```sh
stencila themes tokens --scope semantic --as json
```

## Core semantic starting points

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

These are the stable public API for theme authors. Override them before reaching for node-, plot-, print-, or site-specific tokens.

## Primitive font stacks

Use these when adding custom fonts while preserving fallbacks:

- `--font-family-serif`
- `--font-family-sans`
- `--font-family-mono`
- `--font-family-math`

A typical pattern is to extend a primitive stack and then route it through semantic tokens.

Example:

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
2. Choose the smallest semantic foundation that covers the user’s typography, color, surface, spacing, and layout goals.
3. Use this reference for authoring patterns that the token list does not show, such as fallback-stack extension and the preference for semantic tokens as the stable public API.
4. Move to module-specific scopes only when semantic tokens are not enough.

## Dark mode variants

Many semantic tokens have `*-dark` variants that are applied automatically via `prefers-color-scheme: dark`. Key dark-variant tokens include:

- `--text-color-primary-dark`
- `--color-accent-dark`
- `--surface-background-dark`
- `--surface-elevated-dark`

Override `*-dark` variants explicitly when the light-mode value does not work well on a dark background. If a value is suitable for both schemes, overriding only the light-mode token is sufficient. Use `stencila themes tokens --scope semantic` to see which tokens have dark variants.

## Authoring rules

- Define `@font-face` or external `@import` font declarations before overriding the related font tokens.
- Prefer extending fallback stacks instead of replacing them with a single family name.
- Keep exportable cross-target choices at top-level `:root`.
- If the request is only strategic, recommend token families and fallback-stack patterns without inventing asset files.

## Verified source basis

Localized from the Stencila theme overview and fonts token documentation when this skill was prepared.
