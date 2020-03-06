import { translate } from '../../util'

export interface ThemeObject {
  [key: string]: string
}

/**
 * The keys used to refer to which example and
 * which theme the user wants to see.
 */
export enum keys {
  EXAMPLE = 'example',
  THEME = 'theme',
  HEADER = 'header'
}

/**
 * Create an object consisting only of changed values
 *
 * @function diff
 * @param {Object} original - Source object to compare against
 * @param {Object} updated - New object with partially updated values
 * @return {Object} Object containing keys with changed values
 */
export const diff = <O extends Record<string, unknown>>(
  original: O,
  updated: O
): O => {
  return Object.entries(updated).reduce((_diff: O, [name, value]) => {
    return value === original[name]
      ? _diff
      : { ..._diff, [name]: value === '' ? original[name] : value }
  }, {} as O)
}

/**
 * Convert a JS object to a stringified CSS rule, using object keys as variable names.
 *
 * @function objToVars
 * @param {Object} obj - JS Object, where the keys are variable names (without leading `--` dashes)
 * @param {boolean} [compile] - If true, translates Stencila's Custom `:--root` Selector into `[data-itemscope='root']`
 * @return {string} Stringified CSS styleesheet containing variable declarations
 */
export const objToVars = (obj: ThemeObject, compile = false): string => {
  const vars = Object.entries(obj).reduce(
    (vs: string, [name, value]) => vs + `  --${name}: ${value};\n`,
    ''
  )

  return `${compile ? ':--root' : translate(':--root')} {
${vars}}`
}

/**
 * Submit a PR to https://github.com/stencila/thema for a new theme.
 *
 * Uses the github `/new` route to create a new `styles.css` file
 * within `src/themes/<new-theme-name>` folder.
 *
 * This seems to be by far the easiest way to create a pre-populated pull request
 * for the current Github user (they get redirected automatically to create a fork etc).
 * The only issue is that this only creates a single file, when, currently, we also need a `README.md`
 * and a `index.ts` file. I propose to change our theme build process to not require these,
 * e.g. (create a `index.ts` if necessary; read description from `styles.css` instead of
 * fom `README.md`). The alternative is to have a PR bot that does that - but that
 * feels overly complicated.
 *
 * To reduce friction, the user should be able to push the "create PR" button without having to
 * think about a name or a description. We all know how hard, naming is, so how
 * about pre-populating it, like Github suggests a name for Github repos.
 */
export const submitPR = (
  name: string,
  desc: string,
  theme: ThemeObject,
  baseName: string,
  baseTheme: ThemeObject
): void => {
  // Provide default values where user did not provide any
  name = name.length > 0 ? name : 'randomname'
  // desc = desc.length > 0 ? desc : 'Please provide a description of your theme'

  const diffs = diff(baseTheme, theme)
  const customisations =
    Object.keys(diffs).length === 0
      ? '/* No changes were made to variables in the base theme but you can set them here if you like :) */\n'
      : objToVars(diffs, false)

  const css = `@import "../${baseName}/styles.css";\n\n${customisations}\n`
  const value = encodeURIComponent(css)
  const url = `https://github.com/stencila/thema/new/master?filename=src/themes/${name}/styles.css&value=${value}`
  const win = window.open(url, '_blank')
  if (win !== null) win.focus()
}
