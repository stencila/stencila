import 'construct-style-sheets-polyfill'
import { Configuration, create, cssomSheet } from 'twind'
import * as colors from 'twind/colors'

const twConfig: Configuration = {
  theme: {
    extend: {
      colors: {
        emerald: colors.emerald,
        slate: colors.blueGray,
        violet: colors.violet,
      },
    },
  },
}

/**
 * Create a [Constructable Stylesheet](https://web.dev/constructable-stylesheets/)
 * using `twind`.
 *
 * Usage is usually something like,
 *
 * ```js
 * const { tw, sheet } = twSheet()
 *
 * @customElement('my-custom-element')
 * export default class MyCustomElement extends StencilaElement {
 *   static styles = [sheet.target]
 * ```
 *
 * See the [`twind` docs](https://twind.dev/usage-guides/lit-element.html) for more.
 */
export function twSheet() {
  const sheet = cssomSheet({ target: new CSSStyleSheet() })
  const { tw } = create({ ...twConfig, sheet })
  return { tw, sheet }
}

/**
 * Names, target property, and defaults of CSS variables used for theming
 *
 * Variables prefixed with `main-` are used for styling content. Variables
 * starting with `ui-` are used for styling content in user interfaces,
 * specifically in Web Components generated around and withing the main content.
 */
export const vars = {
  'main-max-width': ['max-width', '70ch'],

  'main-margin-top': ['margin-top', 'auto'],
  'main-margin-bottom': ['margin-bottom', 'auto'],
  'main-margin-left': ['margin-left', 'auto'],
  'main-margin-right': ['margin-right', 'auto'],

  'main-padding-top': ['padding-top', '2ch'],
  'main-padding-bottom': ['padding-bottom', '2ch'],
  'main-padding-left': ['padding-left', '1ch'],
  'main-padding-right': ['padding-right', '1ch'],

  'main-background-color': ['background-color', '#fcfcfc'],

  'main-font-family': [
    'font-family',
    'Palatino, "Book Antiqua", Georgia, serif',
  ],
  'main-font-size': ['font-size', '120%'],
  'main-line-height': ['line-height', 1.5],

  'main-text-color': ['color', '#3a3a3a'],
  'main-heading-color': ['color', '#3e3e3e'],
  'main-paragraph-color': ['color', 'var(--stencila-main-text-color, #3a3a3a)'],
  'main-strong-color': ['color', '#3e3e3e'],

  'main-heading-margin-top': ['margin-top', '2ch'],
  'main-heading-margin-bottom': ['margin-bottom', '1ch'],

  'ui-border-style': ['border-style', 'solid'],
  'ui-border-width': ['border-width', '1px'],
  'ui-border-color': ['border-color', 'var(--stencila-color-neutral-200)'],
  'ui-border-radius': ['border-radius', '3px'],

  'ui-background-color': [
    'background-color',
    'var(--stencila-color-neutral-100)',
  ],

  'ui-icon-color': ['color', 'var(--stencila-color-neutral-300)'],

  'ui-font-family': [
    'font-family',
    'ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "Noto Sans", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji"',
  ],
  'ui-font-size': ['font-size', '1rem'],
  'ui-text-color': ['color', 'var(--stencila-color-neutral-400)'],
}

/**
 * Reference a global value for a variable with a fallback to the
 * default defined in `vars`
 */
export function varGlobal(name: string) {
  return `var(--stencila-${name}, ${vars[name]?.[1] ?? 'none'})`
}

export function varUse(...names: string[]) {
  return names
    .map((name) => `${vars[name]?.[0] ?? name}: ${varGlobal(name)};`)
    .join('')
}

/**
 * Create a local CSS variable (with the `--local` suffix) with a value
 * that can be set (e.g. using `varPass`) but falls back to the global value
 */
export function varLocal(...names: string[]) {
  return names
    .map((name) => `--${name}-local: var(--${name}, ${varGlobal(name)});`)
    .join('')
}

/**
 * Apply a local CSS variable by setting its CSS target property to its value
 */
export function varApply(...names: string[]) {
  return names
    .map((name) => `${vars[name]?.[0] ?? name}: var(--${name}-local);`)
    .join('')
}

/**
 * Pass a local CSS variable to another custom element so that it inherits the
 * current value
 */
export function varPass(...names: string[]) {
  return names.map((name) => `--${name}: var(--${name}-local);`).join('')
}

/**
 * Add a CSS stylesheet to the document
 */
export function addStylesheet(css: string) {
  const sheet = new CSSStyleSheet()
  sheet.replaceSync(css)
  document.adoptedStyleSheets = [...document.adoptedStyleSheets, sheet]
}
