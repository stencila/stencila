import { themes } from '../themes/index'

export { themes }

export interface ThemaAssets {
  styles: string[]
  scripts: string[]
}

const themaMajor = process.env.npm_package_version?.split('.')[0]

/**
 * The path to a theme in this package
 */
export const themePath = `dist/themes`

/**
 * The conventional name for theme stylesheets
 */
export const styleEntry = 'styles.css'

/**
 * The conventional name for theme JavaScript
 */
export const scriptEntry = 'index.js'

export type ThemeNames = keyof typeof themes

/**
 * Tests whether a given string is a valid Thema theme or not.
 *
 * @param {string} name Name of the theme
 */
export const isTheme = (theme: string): theme is ThemeNames =>
  Object.keys(themes).includes(theme)

/**
 * Return a CDN link to an asset, cleaning up any Windows specific path separators.
 */
export const generateCDNUrl = (asset: string): string => {
  return `https://unpkg.com/@stencila/thema@${
    themaMajor ?? 'latest'
  }/${asset}`.replace(/\\/g, '/')
}

/**
 * Given a string, will return a matching theme assets, relative to the project root,
 * returns undefined if a theme cannot be found.
 *
 * @param {string} theme - Name of the theme to look for
 * @param {boolean | undefined} asCDNUrl - If true, returns the assets as URLs pointing to UNPKG hosted files.
 * @return {ThemaAssets | undefined} Object containing two arrays, one of all the themes stylesheets, and one of all
 * scripts.
 */

export const getTheme = (
  theme: string,
  asCDNUrl: boolean | undefined = false
): ThemaAssets | undefined => {
  const _theme = theme.trim().toLowerCase()

  if (!isTheme(_theme)) return undefined

  const style = `${themePath}/${_theme}/${styleEntry}`
  const script = `${themePath}/${_theme}/${scriptEntry}`

  // Return either the filepath or a link to the CDN hosted file
  const resolve = (assets: string[]): string[] =>
    asCDNUrl ? assets.map(generateCDNUrl) : assets

  return {
    styles: resolve([style]),
    scripts: resolve([script]),
  }
}
