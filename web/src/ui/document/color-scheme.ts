/**
 * Color scheme preferences for the document
 */
export type ColorScheme = 'system' | 'light' | 'dark'

/**
 * Utility functions for color scheme management
 */
export class ColorSchemeManager {
  // Set up media query listener for system color scheme changes
  // @ts-expect-error is declared but its value is never read.
  private static mediaQueryListener = (() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => {
      // Only dispatch event if using system preference
      const currentPreference = ColorSchemeManager.loadColorSchemePreference()
      if (currentPreference === 'system') {
        window.dispatchEvent(new CustomEvent('stencila-color-scheme-changed'))
      }
    }
    mediaQuery.addEventListener('change', handleChange)
    return mediaQuery
  })()

  static loadColorSchemePreference(): ColorScheme {
    try {
      const saved = localStorage.getItem('stencila-color-scheme-preference') as ColorScheme
      return saved && ['system', 'light', 'dark'].includes(saved) ? saved : 'system'
    } catch {
      return 'system'
    }
  }

  static applyColorScheme(colorScheme: ColorScheme) {
    const root = document.documentElement

    if (colorScheme === 'system') {
      root.removeAttribute('data-color-scheme')
    } else {
      root.setAttribute('data-color-scheme', colorScheme)
    }

    // Dispatch event to notify components of color scheme change
    window.dispatchEvent(new CustomEvent('stencila-color-scheme-changed'))
  }

  static persistColorScheme(colorScheme: ColorScheme) {
    try {
      localStorage.setItem('stencila-color-scheme-preference', colorScheme)
    } catch (e) {
      console.warn('Could not persist color-scheme preference:', e)
    }
  }
}

// Apply saved color scheme immediately to avoid flash
ColorSchemeManager.applyColorScheme(ColorSchemeManager.loadColorSchemePreference())
