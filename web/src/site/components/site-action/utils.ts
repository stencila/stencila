/**
 * Shared utility functions for site action components
 */

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
 * Check if this is a Stencila-hosted site (*.stencila.site or localhost)
 * OAuth only works on Stencila-hosted sites
 */
export function isStencilaHosted(): boolean {
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
 * Check if dev mode is enabled for a specific action.
 * Enable by adding ?{actionId}DevMode=true to the URL or setting localStorage.
 *
 * @param actionId - The action identifier (e.g., 'review', 'upload')
 */
export function isDevMode(actionId: string): boolean {
  if (!isLocalhost()) return false

  const urlParams = new URLSearchParams(window.location.search)
  if (urlParams.get(`${actionId}DevMode`) === 'true') return true
  if (localStorage.getItem(`stencila-${actionId}-dev-mode`) === 'true')
    return true

  return false
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
 * Generate a unique ID
 */
export function generateId(): string {
  return Math.random().toString(36).substring(2, 11)
}

/**
 * Build the sign-in URL for Stencila authentication
 * @param returnUrl - URL to return to after sign-in
 */
export function buildSignInUrl(returnUrl: string = window.location.href): string {
  const encodedReturn = encodeURIComponent(returnUrl)
  return `https://stencila.cloud/sign-in?returnUrl=${encodedReturn}`
}

/**
 * Build the GitHub OAuth connect URL
 * @param returnUrl - URL to return to after connecting
 */
export function buildGitHubConnectUrl(
  returnUrl: string = window.location.href
): string {
  const encodedReturn = encodeURIComponent(returnUrl)
  return `${GITHUB_OAUTH_URL}?returnUrl=${encodedReturn}`
}
