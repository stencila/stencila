/**
 * Available CSS themes for the document
 */
export type Theme = 'stencila' | 'tufte' | 'latex' | 'workspace' | 'user'

/**
 * Utility functions for theme management
 */
export class ThemeManager {
  static loadThemePreference(): Theme {
    try {
      // First check if there's a saved preference
      const saved = localStorage.getItem('stencila-theme-preference') as Theme
      if (saved && ['stencila', 'tufte', 'latex', 'workspace', 'user'].includes(saved)) {
        return saved
      }

      // Otherwise, check the initial theme type from server-rendered meta tag
      const initialThemeType = document.querySelector('meta[name="stencila-initial-theme-type"]')?.getAttribute('content')

      if (initialThemeType === 'user') {
        return 'user'
      } else if (initialThemeType === 'workspace') {
        return 'workspace'
      } else if (initialThemeType === 'builtin') {
        // For builtin themes, get the actual theme name
        const themeName = document.querySelector('meta[name="stencila-initial-theme-name"]')?.getAttribute('content')
        if (themeName && ['stencila', 'tufte', 'latex'].includes(themeName)) {
          return themeName as Theme
        }
      }

      return 'stencila'
    } catch {
      return 'stencila'
    }
  }

  static applyTheme(theme: Theme) {
    const themeLink = document.querySelector('link[data-theme-link]') as HTMLLinkElement
    const themeStyle = document.querySelector('style[data-theme-style]') as HTMLStyleElement

    if (!themeLink) {
      console.warn('Theme link element not found')
      return false
    }

    // Handle custom themes (workspace/user)
    if (theme === 'workspace' || theme === 'user') {
      if (!themeStyle) {
        console.warn('Custom theme style element not found')
        return false
      }

      // Enable the custom style tag, disable the link
      themeStyle.disabled = false
      themeLink.disabled = true

      // Dispatch event
      window.dispatchEvent(new CustomEvent('stencila-theme-changed'))
      return true
    }

    // Handle builtin themes
    const currentHref = themeLink.href
    const lastSlashIndex = currentHref.lastIndexOf('/')
    const baseUrl = currentHref.substring(0, lastSlashIndex + 1)
    const newHref = `${baseUrl}${theme}.css`

    // Disable custom style if present
    if (themeStyle) {
      themeStyle.disabled = true
    }

    // Enable the link
    themeLink.disabled = false

    // Store current href as fallback
    const fallbackHref = currentHref

    // Listen for successful load and dispatch event
    const handleLoad = () => {
      window.dispatchEvent(new CustomEvent('stencila-theme-changed'))
      themeLink.removeEventListener('load', handleLoad)
    }

    // Listen for load error and fallback
    const handleError = () => {
      console.warn(`Theme '${theme}' failed to load, falling back to previous theme`)
      themeLink.href = fallbackHref
      themeLink.removeEventListener('error', handleError)
      themeLink.removeEventListener('load', handleLoad)
    }

    themeLink.addEventListener('load', handleLoad, { once: true })
    themeLink.addEventListener('error', handleError, { once: true })
    themeLink.href = newHref
    return true
  }

  static persistTheme(theme: Theme) {
    try {
      localStorage.setItem('stencila-theme-preference', theme)
    } catch (e) {
      console.warn('Could not persist theme preference:', e)
    }
  }

  /**
   * Apply theme CSS directly without switching files
   * Used for live theme reloading of workspace/user themes
   */
  static updateThemeCSS(css: string) {
    const themeStyle = document.querySelector('style[data-theme-style]') as HTMLStyleElement
    if (!themeStyle) {
      console.warn('Theme style element not found')
      return false
    }

    // Update the style content
    themeStyle.textContent = css

    // Ensure it's enabled
    themeStyle.disabled = false

    // Disable the theme link
    const themeLink = document.querySelector('link[data-theme-link]') as HTMLLinkElement
    if (themeLink) {
      themeLink.disabled = true
    }

    // Dispatch change event
    window.dispatchEvent(new CustomEvent('stencila-theme-changed'))
    return true
  }
}

// Apply saved theme immediately to avoid flash
ThemeManager.applyTheme(ThemeManager.loadThemePreference())
