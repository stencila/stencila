import fs from 'fs'
import path from 'path'
import { themes } from '../themes'
import {
  isTheme,
  generateCDNUrl,
  themePath,
  styleEntry,
  scriptEntry,
  ThemaAssets,
  getTheme as getThemeBrowser
} from '../browser'

export { themes }
export {
  isTheme,
  generateCDNUrl,
  themePath,
  styleEntry,
  scriptEntry,
  ThemaAssets
}

/**
 * Given a string, will return a matching theme assets, either as a link to CDN hosted files,
 * or the stringified file contents.
 * returns undefined if a theme cannot be found.
 *
 * @param {string} theme - Name of the theme to look for
 * @param {boolean | undefined} asCDNUrl - If true, returns the assets as URLs pointing to UNPKG hosted files.
 * @return {ThemaAssets|undefined} Object containing two arrays, one of all the themes stylesheets, and one of all
 * scripts.
 */

export const getTheme = (
  theme: string,
  asCDNUrl: boolean | undefined = false
): ThemaAssets | undefined => {
  const resolvedTheme = getThemeBrowser(theme, asCDNUrl)
  if (resolvedTheme === undefined) return undefined

  // If requesting URLs to CDN hosted files, browser specific function
  // has already returned the links, so we can terminate early.
  if (asCDNUrl) return resolvedTheme

  // Otherwise return file contents
  const readThemeFiles = (assets: string[]): string[] =>
    assets.map(asset =>
      fs
        .readFileSync(
          path.join(__dirname, '..', '..', asset.replace('/', path.sep))
        )
        .toString()
    )

  return {
    styles: readThemeFiles(resolvedTheme.styles),
    scripts: readThemeFiles(resolvedTheme.scripts)
  }
}
