import 'formdata-polyfill'
import { create, translate, append } from '../util'

interface O {
  [key: string]: string
}

export const parseForm = (form: HTMLFormElement): O => {
  const data = new FormData(form)

  return [...data].reduce(
    (data, [name, value]) => ({ ...data, [name]: value }),
    {}
  )
}

/**
 * Create an object consisting only of changed values
 *
 * @function diff
 * @param {O} original - Source object to compare against
 * @param {O} updated - New object with partially updated values
 * @return {O} Object containing keys with changed values
 */
const diff = (original: O, updated: O): O => {
  return Object.entries(updated).reduce((_diff: O, [name, value]) => {
    return value === original[name]
      ? _diff
      : { ..._diff, [name]: value === '' ? original[name] : value }
  }, {})
}

/**
 * Convert a JS object to a stringified CSS rule, using object keys as variable names.
 */
const objToVars = (obj: O): string => {
  const vars = Object.entries(obj).reduce(
    (vars: string, [name, value]) => vars + `--${name}: ${value};\n`,
    ''
  )

  return `${translate(':--root')} {
${vars}}`
}

export const updateTheme = (form: HTMLFormElement, originalTheme: O): void => {
  // Get values from sidebar, find changed values, and convert to CSS rule definition
  const vars = parseForm(form)
  const cssVars = objToVars(diff(originalTheme, vars))

  // Inject updated rules into preview `head` element
  const preview = document.getElementsByTagName('iframe')[0]
  const head = preview.contentDocument?.getElementsByTagName('head')[0]

  if (head != null) {
    // Remove old updated styles if they exist
    const oldStyles = head.querySelector('.varOverrides')
    if (oldStyles !== null) {
      oldStyles.remove()
    }

    const style = create('style', cssVars)
    style.classList.add('varOverrides')

    append(head, style)
  }
}

export const handleVariableChange = (
  input: HTMLInputElement,
  cb: () => unknown
): void => {
  input.addEventListener('keyup', e => {
    e.preventDefault()
    cb()
  })
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
  form: HTMLFormElement,
  baseName: string,
  baseTheme: O
): void => {
  // Provide default values where user did not provide any
  name = name.length > 0 ? name : 'randomname'
  desc = desc.length > 0 ? desc : 'Please provide a description of your theme'

  const vars = parseForm(form)
  const diffs = diff(baseTheme, vars)
  const customisations =
    Object.keys(diffs).length === 0
      ? '  /* No changes were made to variables in the base theme but you can set them here if you like :) */\n'
      : Object.entries(diffs).reduce(
          (vars: string, [name, value]) => vars + `  --${name}: ${value};\n`,
          ''
        )
  const css = `/*\n${desc}\n*/\n\n@import "../${baseName}/styles.css";\n\n:--root {\n${customisations}}\n`
  const value = encodeURIComponent(css)
  const url = `https://github.com/stencila/thema/new/master?filename=src/themes/${name}/styles.css&value=${value}`
  const win = window.open(url, '_blank')
  if (win !== null) win.focus()
}
