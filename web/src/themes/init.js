/**
 * Color scheme initialization script to prevent flash of incorrect theme.
 *
 * This script MUST be included in the HTML <head> BEFORE any CSS imports
 * to prevent the flash of incorrect theme when users have manually overridden
 * their system preference.
 */
(function () {
  'use strict'

  try {
    const saved = localStorage.getItem('stencila-color-scheme-preference')
    if (saved && (saved === 'light' || saved === 'dark')) {
      document.documentElement.setAttribute('data-color-scheme', saved)
    }
    // If saved is 'system' or null, let CSS media queries handle it naturally
  } catch (error) {
    // localStorage might not be available (file:// protocol, privacy mode, etc.)
    // Silently fail and let CSS media queries determine the theme
  }
})()
