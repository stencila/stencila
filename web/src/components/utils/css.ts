import { create, cssomSheet } from 'twind'

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
  const { tw } = create({ sheet })
  return { tw, sheet }
}

/**
 * Names, target property, and defaults of CSS variables used for theming
 */
const vars = {
  'border-style': ['border-style', 'solid'],
  'border-width': ['border-width', '1px'],
  'border-color': ['border-color', '#eee'],
  'border-radius': ['border-radius', '3px'],

  'bg-color': ['background-color', 'none'],

  'icon-color': ['color', '#ddd'],

  'text-font': ['font-family', 'sans'],
  'text-size': ['font-size', '1rem'],
  'text-color': ['color', '#ccc'],
}

/**
 * Reference a global value for a variable with a fallback to the
 * default defined in `vars`
 */
export function varGlobal(name: string) {
  return `var(--stencila-${name}, ${vars[name]?.[1] ?? 'none'})`
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
