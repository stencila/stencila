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
