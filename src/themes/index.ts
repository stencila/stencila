import { themes } from './themes'

export { themes }

/**
 * The path to a theme in this package
 */
export const themePath = 'dist/themes'

/**
 * Is the string a theme name?
 *
 * @param {string} name Name of the theme
 */
export const isTheme = (name: string): name is keyof typeof themes =>
  name in themes

/**
 * Given a string, will return a matching theme,
 * falling back to the first in if none matches.
 *
 * @param {string} name Name of the theme to look for
 */
export const resolveTheme = (name?: string): string => {
  const theme = name === undefined ? '' : name.toLowerCase().trim()
  return theme !== 'skeleton' && isTheme(theme) ? theme : 'stencila'
}
