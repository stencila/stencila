import { themes } from './themes'

export { themes }

export interface ThemaAssets {
  styles: string[]
  scripts: string[]
}

const themaVersion = process.env.VERSION ?? '1'
const themaMajor = themaVersion.split('.')[0]

/**
 * The path to a theme in this package
 */
const themePath = 'dist/themes'

/**
 * The conventional name for theme stylesheets
 */
export const styleEntry = 'styles.css'

/**
 * The conventional name for theme JavaScript
 */
export const scriptEntry = 'index.js'

/**
 * Tests whether a given string is a valid Thema theme or not.
 *
 * @param {string} name Name of the theme
 */
export const isTheme = (theme: string): theme is keyof typeof themes =>
  Object.keys(themes).includes(theme.toLowerCase().trim())

/**
 * Return a CDN link to an asset, cleaning up any Windows specific path separators.
 */
export const generateCDNUrl = (asset: string): string => {
  return `https://unpkg.com/@stencila/thema@${themaMajor}/${asset}`.replace(
    /\\/g,
    '/'
  )
}

/**
 * Given a string, will return a matching theme assets, relative to the project root,
 * falling back to `stencila` if none matches.
 *
 * @param {string} name Name of the theme to look for
 */
export const resolveTheme = (
  theme: string,
  asCDNUrl: boolean
): ThemaAssets | undefined => {
  if (!isTheme(theme)) return undefined

  const style = `${themePath}/${theme}/${styleEntry}`
  const script = `${themePath}/${theme}/${scriptEntry}`

  // Return either the filepath or a link to the CDN hosted file
  const resolve = (assets: string[]): string[] =>
    asCDNUrl ? assets.map(generateCDNUrl) : assets

  return {
    styles: resolve([style]),
    scripts: resolve([script])
  }
}
