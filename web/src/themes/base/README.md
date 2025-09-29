# Guidelines for developing base theme

## Token architecture

The design system uses a three-tier token architecture that separates concerns and provides automatic theming behavior:

**Primitives** (`tokens-primitive.css`) → **Semantics** (`tokens-semantic.css`) → **Components**

- **Primitive tokens**: Raw foundation values like `--space-4: 1rem`, `--color-gray-800: #262626`, `--font-family-serif`. No variants or contextual meaning. Override with caution only for system-wide changes (e.g., custom web fonts).
- **Semantic tokens**: Meaningful aliases like `--content-spacing: var(--space-8)`, `--text-color-primary: var(--color-gray-800)` that reference primitives using `var()`. These provide automatic dark mode, responsive, and print behavior through suffix variants (-dark, -tablet, -mobile, -print).
- **Component tokens**: Component-specific tokens like `--heading-spacing-top: calc(var(--content-spacing) * 1.5)` that reference semantic tokens to inherit automatic theming behavior.

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

/* ❌ Bad - Fighting against semantic token scaling */
@media (max-width: 640px) {
  :root {
    --list-spacing: var(--space-2); /* Breaks content-spacing relationship */
  }
}

/* ❌ Bad - Redundant print overrides */
@media print {
  :root {
    --list-spacing: var(--space-1); /* content-spacing already handles print */
  }
}
```

## Token Relationships and Best Practices

When defining component tokens, consider their relationship to semantic tokens for better integration and automatic responsive behavior. Build token hierarchies where related tokens derive from the same semantic base:

**Spacing tokens** should typically base themselves on `--content-spacing` or its multiples to ensure automatic responsive scaling:

```css
/* ✅ Good - Inherits responsive behavior automatically */
:root {
  --component-spacing-top: calc(var(--content-spacing) * 1.5);
}

/* ✅ Good - Build token hierarchies from the same base */
:root {
  --list-spacing: var(--content-spacing);
  --list-item-spacing: calc(var(--list-spacing) * 0.25);
  --list-indent: calc(var(--content-spacing) * 0.75);
}

/* ⚠️ Consider carefully - May need manual responsive variants */
:root {
  --component-spacing-top: var(--space-12);
  --component-spacing-top-tablet: var(--space-8);
  --component-spacing-top-mobile: var(--space-6);
}
```

**Color tokens** should reference semantic color tokens that already include dark variants:

```css
/* ✅ Good - Gets dark mode automatically */
:root {
  --component-background: var(--surface-foreground);
  --component-text: var(--text-color-primary);
}
```

**Typography tokens** should use semantic font and line-height tokens for consistency:

```css
/* ✅ Good - Follows document typography system */
:root {
  --component-font-family: var(--text-font-family);
  --component-line-height: var(--line-height-xs);
}
```

**Additional specialized token files**:

- **Page layout tokens** (`tokens-page.css`): Defines tokens for paged media layout (@page rules), including page margins, headers, footers, and print-specific page styling. These tokens control how documents appear when printed or exported to PDF.
- **User agent tokens** (`tokens-ua.css`): Provides browser UI normalizations for cross-browser consistency. Currently includes scrollbar styling tokens, with potential for future additions like form controls, focus rings, and selection colors.

## Structure of node-specific CSS files

Most of the files in this folder are for node type specific styles (e.g. `admonition.css` and `headings.css`). These files should follow the following structure:

### 1. Header comment

Each CSS module must start with a comprehensive documentation block that provides developers with complete context for styling Stencila node types. The header includes schema links, DOM structure examples, key attributes, slots, and usage notes to help developers understand the nested custom element architecture and write effective CSS rules. Follow the structure and format shown in existing files like `admonitions.css` and `math.css` for consistency.

### 2. Component design tokens

Define CSS custom properties (design tokens) in the `:root` selector using a consistent naming convention: `--component-property-modifier` (e.g., `--heading-font-size`, `--table-border-color`). Reference existing base tokens from the design system where possible to maintain consistency across the theme.

**Token organization:** Group related tokens with descriptive comments to improve maintainability.

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

Responsive variants are optional and should be used judiciously. Consider whether your component needs explicit responsive behavior or can inherit it from semantic tokens:

**When to use explicit responsive variants:**

- Component-specific adjustments that don't scale proportionally with document spacing
- Override cases where automatic scaling doesn't work well (e.g., mobile h1 margins)
- Typography that needs different scaling than the document default

**When to rely on semantic token inheritance (preferred):**

- Spacing that should scale with document rhythm (base on `--content-spacing`)
- Typography that follows document-wide responsive adjustments
- Colors and other properties that don't need responsive changes
- Print layouts that should scale with document print spacing
- Any component behavior that should follow system-wide theming

**Automatic scaling through semantic tokens:**

Components that use `--content-spacing` automatically inherit responsive and print behavior without explicit media queries. For example, lists using `--list-spacing: var(--content-spacing)` automatically scale on mobile (`--content-spacing-mobile`) and print (`--content-spacing-print`) without additional CSS.

If responsive variant tokens are defined, apply them using CSS fallbacks to create a cascade from desktop to tablet to mobile:

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

#### Special Cases and Overrides

When adding responsive overrides that deviate from automatic scaling or token inheritance, always document why they exist:

```css
/* Mobile breakpoint (640px) - Special case for h1 top margin
 * Override from automatic 1.5rem (calc(1rem * 1.5)) to 1rem for better mobile spacing */
@media (max-width: 640px) {
  stencila-heading {
    h1 {
      margin-top: var(--space-4);
    }
  }
}
```

If you add CSS rules that aim for normalization across browsers, add a comment `/* Browser normalization */`. This helps other developers understand why a particular rule exists.

## Documentation

The theming system and its design tokens are documented in `docs/themes`. Node-specific design tokens have their own documentation file e.g. `paragraph.smd`. Follow the structure and style of these when creating new documentation.

Most of the documentation file include examples to demonstrate token usage. When writing these:

**Keep examples focused**: Each example should demonstrate specific token usage patterns rather than complex layouts

**Prefer design tokens over raw values**: Use `var(--space-10)` instead of `2.5rem`, `var(--font-size-lg)` instead of `1.125rem`

**Use StyledBlocks for demonstrations**: Pair a CSS `CodeBlock` with a `StyledBlock` for clean, readable examples that apply token overrides directly to rendered content:

````markdown
```css
:root {
  --component-spacing: var(--space-4);
  --component-color: var(--color-accent);
}
```

::: style --component-spacing: var(--space-4); --component-color: var(--color-accent);

Your Stencila Markdown example here

:::
````

**Test token combinations**: Ensure the token values used in examples actually work together harmoniously

## Component CSS Checklist

Before committing a component CSS file, verify:

- [ ] All referenced tokens exist in primitive or semantic token layers
- [ ] Component tokens follow `--component-property-modifier` naming convention
- [ ] Dark mode variants are defined and applied where needed using the standard pattern
- [ ] Responsive behavior is either inherited from semantic tokens or explicitly defined with variants
- [ ] Special cases and responsive overrides are documented with explanatory comments
- [ ] Browser normalizations are marked with `/* Browser normalization */` comments
- [ ] File follows the standard structure: header comment, component tokens, dark mode rules, responsive rules
- [ ] Component tokens leverage semantic tokens where possible for automatic theming behavior
- [ ] Spacing tokens are based on `--content-spacing` unless component-specific spacing is required
- [ ] Documentation exists in `docs/themes/` following the established pattern and is accurate with respect to the implementation
