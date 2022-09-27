/**
 * Base theme
 *
 * This module dynamically generates CSS designed to be used as a base for themes.
 * It defines a large number of CSS variables that can be used by theme.
 * It then uses those variables in a very simple stylesheet. The intention is that
 * themes can largely be generated just by setting variables with extra CSS rules where
 * necessary.
 */

import { addStylesheet, varGlobal, vars, varUse } from '../components/utils/css'

// Create variables used in main content and in Web Components
const stencilaGlobalVariables = Object.entries(vars)
  .map(([name, [target, value]]) => `--stencila-${name}: ${value};`)
  .join('\n')

// Create Stencila literal color variables using Tailwind's color pallette (excluding 'neutral' which is overloaded by
// our semantic color name). From  https://github.com/tailwindlabs/tailwindcss/blob/b8cda161dd0993083dcef1e2a03988c70be0ce93/src/public/colors.js
const stencilaLiteralColors = Object.entries({
  slate: {
    50: '#f8fafc',
    100: '#f1f5f9',
    200: '#e2e8f0',
    300: '#cbd5e1',
    400: '#94a3b8',
    500: '#64748b',
    600: '#475569',
    700: '#334155',
    800: '#1e293b',
    900: '#0f172a',
  },
  gray: {
    50: '#f9fafb',
    100: '#f3f4f6',
    200: '#e5e7eb',
    300: '#d1d5db',
    400: '#9ca3af',
    500: '#6b7280',
    600: '#4b5563',
    700: '#374151',
    800: '#1f2937',
    900: '#111827',
  },
  zinc: {
    50: '#fafafa',
    100: '#f4f4f5',
    200: '#e4e4e7',
    300: '#d4d4d8',
    400: '#a1a1aa',
    500: '#71717a',
    600: '#52525b',
    700: '#3f3f46',
    800: '#27272a',
    900: '#18181b',
  },
  stone: {
    50: '#fafaf9',
    100: '#f5f5f4',
    200: '#e7e5e4',
    300: '#d6d3d1',
    400: '#a8a29e',
    500: '#78716c',
    600: '#57534e',
    700: '#44403c',
    800: '#292524',
    900: '#1c1917',
  },
  red: {
    50: '#fef2f2',
    100: '#fee2e2',
    200: '#fecaca',
    300: '#fca5a5',
    400: '#f87171',
    500: '#ef4444',
    600: '#dc2626',
    700: '#b91c1c',
    800: '#991b1b',
    900: '#7f1d1d',
  },
  orange: {
    50: '#fff7ed',
    100: '#ffedd5',
    200: '#fed7aa',
    300: '#fdba74',
    400: '#fb923c',
    500: '#f97316',
    600: '#ea580c',
    700: '#c2410c',
    800: '#9a3412',
    900: '#7c2d12',
  },
  amber: {
    50: '#fffbeb',
    100: '#fef3c7',
    200: '#fde68a',
    300: '#fcd34d',
    400: '#fbbf24',
    500: '#f59e0b',
    600: '#d97706',
    700: '#b45309',
    800: '#92400e',
    900: '#78350f',
  },
  yellow: {
    50: '#fefce8',
    100: '#fef9c3',
    200: '#fef08a',
    300: '#fde047',
    400: '#facc15',
    500: '#eab308',
    600: '#ca8a04',
    700: '#a16207',
    800: '#854d0e',
    900: '#713f12',
  },
  lime: {
    50: '#f7fee7',
    100: '#ecfccb',
    200: '#d9f99d',
    300: '#bef264',
    400: '#a3e635',
    500: '#84cc16',
    600: '#65a30d',
    700: '#4d7c0f',
    800: '#3f6212',
    900: '#365314',
  },
  green: {
    50: '#f0fdf4',
    100: '#dcfce7',
    200: '#bbf7d0',
    300: '#86efac',
    400: '#4ade80',
    500: '#22c55e',
    600: '#16a34a',
    700: '#15803d',
    800: '#166534',
    900: '#14532d',
  },
  emerald: {
    50: '#ecfdf5',
    100: '#d1fae5',
    200: '#a7f3d0',
    300: '#6ee7b7',
    400: '#34d399',
    500: '#10b981',
    600: '#059669',
    700: '#047857',
    800: '#065f46',
    900: '#064e3b',
  },
  teal: {
    50: '#f0fdfa',
    100: '#ccfbf1',
    200: '#99f6e4',
    300: '#5eead4',
    400: '#2dd4bf',
    500: '#14b8a6',
    600: '#0d9488',
    700: '#0f766e',
    800: '#115e59',
    900: '#134e4a',
  },
  cyan: {
    50: '#ecfeff',
    100: '#cffafe',
    200: '#a5f3fc',
    300: '#67e8f9',
    400: '#22d3ee',
    500: '#06b6d4',
    600: '#0891b2',
    700: '#0e7490',
    800: '#155e75',
    900: '#164e63',
  },
  sky: {
    50: '#f0f9ff',
    100: '#e0f2fe',
    200: '#bae6fd',
    300: '#7dd3fc',
    400: '#38bdf8',
    500: '#0ea5e9',
    600: '#0284c7',
    700: '#0369a1',
    800: '#075985',
    900: '#0c4a6e',
  },
  blue: {
    50: '#eff6ff',
    100: '#dbeafe',
    200: '#bfdbfe',
    300: '#93c5fd',
    400: '#60a5fa',
    500: '#3b82f6',
    600: '#2563eb',
    700: '#1d4ed8',
    800: '#1e40af',
    900: '#1e3a8a',
  },
  indigo: {
    50: '#eef2ff',
    100: '#e0e7ff',
    200: '#c7d2fe',
    300: '#a5b4fc',
    400: '#818cf8',
    500: '#6366f1',
    600: '#4f46e5',
    700: '#4338ca',
    800: '#3730a3',
    900: '#312e81',
  },
  violet: {
    50: '#f5f3ff',
    100: '#ede9fe',
    200: '#ddd6fe',
    300: '#c4b5fd',
    400: '#a78bfa',
    500: '#8b5cf6',
    600: '#7c3aed',
    700: '#6d28d9',
    800: '#5b21b6',
    900: '#4c1d95',
  },
  purple: {
    50: '#faf5ff',
    100: '#f3e8ff',
    200: '#e9d5ff',
    300: '#d8b4fe',
    400: '#c084fc',
    500: '#a855f7',
    600: '#9333ea',
    700: '#7e22ce',
    800: '#6b21a8',
    900: '#581c87',
  },
  fuchsia: {
    50: '#fdf4ff',
    100: '#fae8ff',
    200: '#f5d0fe',
    300: '#f0abfc',
    400: '#e879f9',
    500: '#d946ef',
    600: '#c026d3',
    700: '#a21caf',
    800: '#86198f',
    900: '#701a75',
  },
  pink: {
    50: '#fdf2f8',
    100: '#fce7f3',
    200: '#fbcfe8',
    300: '#f9a8d4',
    400: '#f472b6',
    500: '#ec4899',
    600: '#db2777',
    700: '#be185d',
    800: '#9d174d',
    900: '#831843',
  },
  rose: {
    50: '#fff1f2',
    100: '#ffe4e6',
    200: '#fecdd3',
    300: '#fda4af',
    400: '#fb7185',
    500: '#f43f5e',
    600: '#e11d48',
    700: '#be123c',
    800: '#9f1239',
    900: '#881337',
  },
})
  .map(([name, variants]) =>
    Object.entries(variants)
      .map(([level, color]) => `--stencila-color-${name}-${level}: ${color};`)
      .join('\n')
  )
  .join('\n')

// Map Stencila literal color variables into Stencila semantic color variables
const stencilaSemanticColors =
  Object.entries({
    primary: 'blue',
    success: 'green',
    warning: 'amber',
    danger: 'red',
    neutral: 'gray',
  })
    .map(([semantic, literal]) =>
      [50, 100, 200, 300, 400, 500, 600, 700, 800, 900]
        .map(
          (level) =>
            `--stencila-color-${semantic}-${level}: var(--stencila-color-${literal}-${level});`
        )
        .join('\n')
    )
    .join('\n') +
  `
--stencila-color-neutral-0: #fff;
--stencila-color-neutral-1000: #000;
`

// Map the Stencila semantic color variables into the equivalent Shoelace color variables
// for consistency in the shandow DOM od Shoelace components
const shoelaceSemanticColors = [
  'primary',
  'success',
  'warning',
  'danger',
  'neutral',
]
  .map((variant) =>
    [50, 100, 200, 300, 400, 500, 600, 700, 800, 900]
      .map(
        (level) =>
          `--sl-color-${variant}-${level}: var(--stencila-color-${variant}-${level});`
      )
      .join('\n')
  )
  .join('\n')

// A variable for the code editor theme intended to be overridden by
// theme authors
const stencilaCodeEditorTheme = '--stencila-code-editor-theme: fooayuLight;'

// Add a stylesheet for all variables
addStylesheet(`
  :root,
  :host {
    ${stencilaGlobalVariables}
    ${stencilaLiteralColors}
    ${stencilaSemanticColors}
    ${shoelaceSemanticColors}
    ${stencilaCodeEditorTheme}
  }
`)

// Add a stylesheet to theme
addStylesheet(`
  :root,
  :host {
    color-scheme: light;

    --sl-border-radius-small: 0.1875rem;
    --sl-border-radius-medium: 0.25rem;
    --sl-border-radius-large: 0.5rem;
    --sl-border-radius-x-large: 1rem;

    --sl-border-radius-circle: 50%;
    --sl-border-radius-pill: 9999px;

    --sl-shadow-x-small: 0 1px 2px hsl(240 3.8% 46.1% / 6%);
    --sl-shadow-small: 0 1px 2px hsl(240 3.8% 46.1% / 12%);
    --sl-shadow-medium: 0 2px 4px hsl(240 3.8% 46.1% / 12%);
    --sl-shadow-large: 0 2px 8px hsl(240 3.8% 46.1% / 12%);
    --sl-shadow-x-large: 0 4px 16px hsl(240 3.8% 46.1% / 12%);

    --sl-spacing-3x-small: 0.125rem;
    --sl-spacing-2x-small: 0.25rem;
    --sl-spacing-x-small: 0.5rem;
    --sl-spacing-small: 0.75rem;
    --sl-spacing-medium: 1rem;
    --sl-spacing-large: 1.25rem;
    --sl-spacing-x-large: 1.75rem;
    --sl-spacing-2x-large: 2.25rem;
    --sl-spacing-3x-large: 3rem;
    --sl-spacing-4x-large: 4.5rem;

    --sl-transition-x-slow: 1000ms;
    --sl-transition-slow: 500ms;
    --sl-transition-medium: 250ms;
    --sl-transition-fast: 150ms;
    --sl-transition-x-fast: 50ms;

    --sl-font-sans: ${varGlobal('ui-font-family')};

    --sl-font-size-2x-small: 0.625rem;
    --sl-font-size-x-small: 0.75rem;
    --sl-font-size-small: 0.875rem;
    --sl-font-size-medium: 1rem;
    --sl-font-size-large: 1.25rem;
    --sl-font-size-x-large: 1.5rem;
    --sl-font-size-2x-large: 2.25rem;
    --sl-font-size-3x-large: 3rem;
    --sl-font-size-4x-large: 4.5rem;

    --sl-font-weight-light: 300;
    --sl-font-weight-normal: 400;
    --sl-font-weight-semibold: 500;
    --sl-font-weight-bold: 700;

    --sl-letter-spacing-denser: -0.03em;
    --sl-letter-spacing-dense: -0.015em;
    --sl-letter-spacing-normal: normal;
    --sl-letter-spacing-loose: 0.075em;
    --sl-letter-spacing-looser: 0.15em;

    --sl-line-height-denser: 1;
    --sl-line-height-dense: 1.4;
    --sl-line-height-normal: 1.8;
    --sl-line-height-loose: 2.2;
    --sl-line-height-looser: 2.6;

    --sl-focus-ring-color: ${varGlobal('color-primary-600')};
    --sl-focus-ring-style: solid;
    --sl-focus-ring-width: 3px;
    --sl-focus-ring: var(--sl-focus-ring-style) var(--sl-focus-ring-width)
      var(--sl-focus-ring-color);
    --sl-focus-ring-offset: 1px;

    --sl-button-font-size-small: var(--sl-font-size-x-small);
    --sl-button-font-size-medium: var(--sl-font-size-small);
    --sl-button-font-size-large: var(--sl-font-size-medium);

    --sl-input-height-small: 1.875rem;
    --sl-input-height-medium: 2.5rem;
    --sl-input-height-large: 3.125rem;

    --sl-input-background-color: ${varGlobal('color-neutral-0')};
    --sl-input-background-color-hover: var(--sl-input-background-color);
    --sl-input-background-color-focus: var(--sl-input-background-color);
    --sl-input-background-color-disabled: ${varGlobal('color-neutral-100')};
    --sl-input-border-color: ${varGlobal('color-neutral-300')};
    --sl-input-border-color-hover: ${varGlobal('color-neutral-400')};
    --sl-input-border-color-focus: ${varGlobal('color-primary-500')};
    --sl-input-border-color-disabled: ${varGlobal('color-neutral-300')};
    --sl-input-border-width: 1px;
    --sl-input-required-content: "*";
    --sl-input-required-content-offset: -2px;

    --sl-input-border-radius-small: var(--sl-border-radius-medium);
    --sl-input-border-radius-medium: var(--sl-border-radius-medium);
    --sl-input-border-radius-large: var(--sl-border-radius-medium);

    --sl-input-font-family: var(--sl-font-sans);
    --sl-input-font-weight: var(--sl-font-weight-normal);
    --sl-input-font-size-small: var(--sl-font-size-small);
    --sl-input-font-size-medium: var(--sl-font-size-medium);
    --sl-input-font-size-large: var(--sl-font-size-large);
    --sl-input-letter-spacing: var(--sl-letter-spacing-normal);

    --sl-input-color: ${varGlobal('color-neutral-700')};
    --sl-input-color-hover: ${varGlobal('color-neutral-700')};
    --sl-input-color-focus: ${varGlobal('color-neutral-700')};
    --sl-input-color-disabled: ${varGlobal('color-neutral-900')};
    --sl-input-icon-color: ${varGlobal('color-neutral-500')};
    --sl-input-icon-color-hover: ${varGlobal('color-neutral-600')};
    --sl-input-icon-color-focus: ${varGlobal('color-neutral-600')};
    --sl-input-placeholder-color: ${varGlobal('color-neutral-500')};
    --sl-input-placeholder-color-disabled: ${varGlobal('color-neutral-600')};
    --sl-input-spacing-small: var(--sl-spacing-small);
    --sl-input-spacing-medium: var(--sl-spacing-medium);
    --sl-input-spacing-large: var(--sl-spacing-large);

    --sl-input-filled-background-color: ${varGlobal('color-neutral-100')};
    --sl-input-filled-background-color-hover: ${varGlobal('color-neutral-100')};
    --sl-input-filled-background-color-focus: ${varGlobal('color-neutral-100')};
    --sl-input-filled-background-color-disabled: ${varGlobal(
      'color-neutral-100'
    )};
    --sl-input-filled-color: ${varGlobal('color-neutral-800')};
    --sl-input-filled-color-hover: ${varGlobal('color-neutral-800')};
    --sl-input-filled-color-focus: ${varGlobal('color-neutral-700')};
    --sl-input-filled-color-disabled: ${varGlobal('color-neutral-800')};

    --sl-input-focus-ring-color: hsl(198.6 88.7% 48.4% / 40%);
    --sl-input-focus-ring-offset: 0;

    --sl-input-label-font-size-small: var(--sl-font-size-small);
    --sl-input-label-font-size-medium: var(--sl-font-size-medium);
    --sl-input-label-font-size-large: var(--sl-font-size-large);

    --sl-input-label-color: inherit;

    --sl-input-help-text-font-size-small: var(--sl-font-size-x-small);
    --sl-input-help-text-font-size-medium: var(--sl-font-size-small);
    --sl-input-help-text-font-size-large: var(--sl-font-size-medium);

    --sl-input-help-text-color: ${varGlobal('color-neutral-500')};

    --sl-toggle-size: 1rem;

    --sl-overlay-background-color: hsl(240 3.8% 46.1% / 33%);

    --sl-panel-background-color: ${varGlobal('color-neutral-0')};
    --sl-panel-border-color: ${varGlobal('color-neutral-200')};
    --sl-panel-border-width: 1px;

    --sl-tooltip-border-radius: var(--sl-border-radius-medium);
    --sl-tooltip-background-color: ${varGlobal('color-neutral-800')};
    --sl-tooltip-color: ${varGlobal('color-neutral-0')};
    --sl-tooltip-font-family: var(--sl-font-sans);
    --sl-tooltip-font-weight: var(--sl-font-weight-normal);
    --sl-tooltip-font-size: var(--sl-font-size-small);
    --sl-tooltip-line-height: var(--sl-line-height-dense);
    --sl-tooltip-padding: var(--sl-spacing-2x-small) var(--sl-spacing-x-small);
    --sl-tooltip-arrow-size: 4px;

    --sl-z-index-drawer: 700;
    --sl-z-index-dialog: 800;
    --sl-z-index-dropdown: 900;
    --sl-z-index-toast: 950;
    --sl-z-index-tooltip: 1000;
  }

  .sl-scroll-lock {
    overflow: hidden !important;
  }

  .sl-toast-stack {
    position: fixed;
    inset-inline-end: 0;
    z-index: var(--sl-z-index-toast);
    max-width: 100%;
    max-height: 100%;
    overflow: auto;

    /* Customized to appear bottom-right */
    bottom: 1rem;
    right: 1rem;
    width: 22rem;
    margin: 2px;
  }

  .sl-toast-stack sl-alert {
    --box-shadow: var(--sl-shadow-large);
    margin: var(--sl-spacing-medium);
  }
`)

// Add a stylesheet with CSS rules for the main content
addStylesheet(`
  body {
    margin: 0;

    ${varUse('main-background-color')}
  }

  main {
    ${varUse(
      'main-max-width',

      'main-margin-top',
      'main-margin-bottom',
      'main-margin-left',
      'main-margin-right',

      'main-padding-top',
      'main-padding-bottom',
      'main-padding-left',
      'main-padding-right',

      'main-font-family',
      'main-font-size',
      'main-line-height',

      'main-text-color'
    )}
  }

  main h1,
  main h2,
  main h3,
  main h4,
  main h5,
  main h6 {
    ${varUse(
      'main-heading-color',
      'main-heading-margin-top',
      'main-heading-margin-bottom'
    )}
  }

  main p {
    ${varUse('main-paragraph-color')}
  }

  main strong {
    ${varUse('main-strong-color')}
  }
`)
