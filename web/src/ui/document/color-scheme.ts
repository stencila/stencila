/**
 * Color scheme preferences for the document
 */
export type ColorScheme = 'light' | 'dark'

/**
 * Utility functions for color scheme management
 */
export class ColorSchemeManager {
  /**
   * Get the system's preferred color scheme
   */
  private static getSystemPreference(): ColorScheme {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  }

  static loadColorSchemePreference(): ColorScheme {
    try {
      const saved = localStorage.getItem('stencila-color-scheme-preference')
      // Handle valid values, migrating 'system' to actual system preference
      if (saved === 'light' || saved === 'dark') {
        return saved
      }
      // First visit or 'system' migration: use system preference
      return ColorSchemeManager.getSystemPreference()
    } catch {
      return ColorSchemeManager.getSystemPreference()
    }
  }

  static applyColorScheme(colorScheme: ColorScheme) {
    document.documentElement.setAttribute('data-color-scheme', colorScheme)
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
