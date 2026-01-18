/**
 * Constants and utility functions for site review components
 */

// Storage keys
export const STORAGE_KEY_ITEMS = 'stencila-site-review-items'
export const STORAGE_KEY_SOURCE = 'stencila-site-review-source'

// API endpoint paths (relative, will be prefixed with apiBase)
export const REVIEW_AUTH_PATH = '/__stencila-review/auth'
export const REVIEW_SUBMIT_PATH = '/__stencila-review/submit'
export const REVIEW_GITHUB_TOKEN_PATH = '/__stencila-review/github-token'

// External URLs
export const GITHUB_OAUTH_URL = 'https://stencila.cloud/github/connect/site'

/**
 * Check if running on localhost
 */
export function isLocalhost(): boolean {
  const hostname = window.location.hostname
  return hostname === 'localhost' || hostname === '127.0.0.1'
}

/**
 * Extract pathname from a URL string
 */
export function getPathname(url: string): string {
  try {
    return new URL(url).pathname
  } catch {
    return url
  }
}

/**
 * Check if this is a Stencila-hosted site (*.stencila.site or localhost)
 * OAuth only works on Stencila-hosted sites
 */
export function isStencilaHostedSite(): boolean {
  const hostname = window.location.hostname
  // *.stencila.site subdomains
  if (hostname.endsWith('.stencila.site')) return true
  // localhost for development
  if (hostname === 'localhost' || hostname === '127.0.0.1') return true
  // Common third-party hosts that won't work
  const thirdPartyHosts = [
    'netlify.app',
    'netlify.com',
    'github.io',
    'vercel.app',
    'pages.dev',
    'surge.sh',
    'render.com',
  ]
  return !thirdPartyHosts.some((host) => hostname.endsWith(host))
}

/**
 * Check if localhost dev mode is enabled.
 * Enable by adding ?reviewDevMode=true to the URL or setting localStorage.
 */
export function isDevMode(): boolean {
  if (!isLocalhost()) return false

  const urlParams = new URLSearchParams(window.location.search)
  if (urlParams.get('reviewDevMode') === 'true') return true
  if (localStorage.getItem('stencila-review-dev-mode') === 'true') return true

  return false
}
