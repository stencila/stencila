# Guidelines for developing base theme

## Token architecture

The design system uses a three-tier token architecture that separates concerns and provides automatic theming behavior:

**Primitives** (`tokens-primitive.css`) → **Semantics** (`tokens-semantic.css`) → **Components**

- **Primitive tokens**: Raw foundation values (colors, fonts, spacing) with no variants. Override with caution only for system-wide changes (e.g., custom web fonts).
- **Semantic tokens**: Theme API that references primitives using `var()`. These provide automatic dark mode and responsive behavior through suffix variants (-dark, -tablet, -mobile).
- **Component usage**: CSS modules use semantic tokens to get automatic theming behavior.

The Semantic tokens are the main public API for theme developers. They should prefer overriding semantic tokens for targeted customization. Override primitives only when you need system-wide changes that affect many tokens at once.

```css
/* ✅ Good - Override semantic tokens for targeted customization */
:root {
  --text-color-primary: var(--color-blue-700);
  --surface-background: var(--color-blue-50);
}

/* ⚠️ Use with caution - Override primitives only for system-wide changes */
:root {
  --font-family-serif: "Custom Font", serif; /* Acceptable for web fonts */
  --color-blue-700: #custom; /* Affects ALL tokens using this primitive */
}

/* ❌ Bad - Don't override primitives for targeted changes */
:root {
  --color-gray-800: #1e40af; /* Use semantic tokens instead */
}
```

## Structure of files

Most of the files in this folder are for node type specific styles (e.g. `admonition.css` and `headings.css`). These files should follow the following structure:

### 1. Header comment

Each CSS module must start with a comprehensive documentation block that provides developers with complete context for styling Stencila node types. The header includes schema links, DOM structure examples, key attributes, slots, and usage notes to help developers understand the nested custom element architecture and write effective CSS rules. Follow the structure and format shown in existing files like `admonitions.css` and `math.css` for consistency.

### 2. Component design tokens

Define CSS custom properties (design tokens) in the `:root` selector using a consistent naming convention: `--component-property-modifier` (e.g., `--heading-font-size`, `--table-border-color`). Reference existing base tokens from the design system where possible to maintain consistency across the theme.

For components requiring dark mode variants, define dark tokens alongside their light counterparts using a `-dark` suffix (e.g., `--table-header-background` and `--table-header-background-dark`). This approach groups related tokens together and makes the light/dark relationship clear.

For responsive design, optionally define `-tablet` and `-mobile` variants that cascade from desktop to mobile (e.g., `--heading-margin-top`, `--heading-margin-top-tablet`, `--heading-margin-top-mobile`). These variants are not required but provide a systematic way to handle responsive adjustments when needed.

### 3. Dark token application

If dark variant tokens were defined, apply them using the following pattern to support both system preference and explicit user choice:

```css
/* Define both light and dark values together */
:root {
  --table-header-background: var(--surface-sunken);
  --table-header-background-dark: var(--color-gray-800);
}

/* Apply dark mode - respects system preference */
@media (prefers-color-scheme: dark) {
  :root:not([data-color-scheme="light"]) {
    --table-header-background: var(--table-header-background-dark);
  }
}

/* Explicit user preference overrides system */
:root[data-color-scheme="dark"] {
  --table-header-background: var(--table-header-background-dark);
}
```

The duplication between the media query and attribute selector is necessary due to CSS limitations - there's no way to combine these conditions into a single rule while maintaining the proper precedence for user preferences.

### 4. Mobile and tablet token application

If responsive variant tokens were defined, apply them using CSS fallbacks to create a cascade from desktop to tablet to mobile:

```css
/* Define responsive variants */
:root {
  --heading-margin-top: var(--space-12);
  --heading-margin-top-tablet: var(--space-8);
  --heading-margin-top-mobile: var(--space-6);
}

/* Tablet breakpoint (768px) */
@media (max-width: 768px) {
  :root {
    --heading-margin-top: var(--heading-margin-top-tablet);
  }
}

/* Mobile breakpoint (640px) */
@media (max-width: 640px) {
  :root {
    --heading-margin-top: var(--heading-margin-top-mobile);
  }
}
```

Each breakpoint simply overrides the base token with its variant. If a variant is not defined, the previous value is maintained through CSS cascade.

### 5. Applying component styles

When writing CSS rules for Stencila components, follow these selector patterns in order of preference:

#### Preferred: Target semantic HTML elements within custom elements

```css
/* ✅ Preferred - Clear hierarchy and semantic meaning */
stencila-heading {
  h1,
  h2,
  h3,
  h4,
  h5,
  h6 {
    font-family: var(--heading-font-family);
    color: var(--heading-color);
  }
}

stencila-paragraph {
  p {
    line-height: var(--paragraph-line-height);
    max-width: var(--content-width);
  }
}
```

This pattern:

- Maintains semantic HTML meaning
- Provides clear component scoping
- Works reliably across all Stencila-generated HTML
- Makes styles easy to understand and debug

#### Occasional: Use slot selectors for specific containers

Use slot selectors when targeting wrapper elements that contain content, applying layout-specific styles to containers, or differentiating between multiple content areas within a component.

#### Avoid: Direct attribute selectors on custom elements

```css
/* ❌ Avoid - Brittle and implementation-dependent */
stencila-heading[level="1"] {
  font-size: 2rem;
}
```

Instead, use the semantic HTML element that corresponds to the attribute (e.g., `stencila-heading h1` rather than `stencila-heading[level="1"]`).

## Notes

- When Stencila generates HTML, it always creates a custom element for the node type (e.g. `<stencila-heading>`) which have attributes and slots containing metadata about that node and which wrap the related semantic HTML element (e.g. `h1`, `h2`). This creates a heavily nested structure which you should be aware of when writing CSS rules.

- If you add CSS rules that aim for normalization across browsers, add a comment `/* Browser normalization */`. This helps other developers understand why a particular rule exists.

- Use CSS custom properties (design tokens) consistently. Token names should follow the pattern `--component-property-modifier` where applicable.
