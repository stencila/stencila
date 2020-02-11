export const themes = {
  elife: 'elife' as const,
  nature: 'nature' as const,
  plos: 'plos' as const,
  stencila: 'stencila' as const
}

export type Themes = typeof themes
export type ThemeNames = Themes[keyof Themes]

export const themePath = 'dist/themes'

const themeNameGuard = (s: string): s is keyof Themes => s in themes

/**
 * Given a string, will attempt to return a matching Thema theme, falling back to Stencila in case none is found
 * @param {string} name Name of the theme to look for
 */
export const getTheme = (name?: string): ThemeNames => {
  const theme = name === undefined ? '' : name.toLowerCase().trim()
  return themeNameGuard(theme) ? themes[theme] : themes.stencila
}
