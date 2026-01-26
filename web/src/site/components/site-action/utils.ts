/**
 * Shared utility functions for site action components
 */

import type { SiteAuthStatusResponse } from './types'

// External URLs
export const GITHUB_OAUTH_URL = 'https://stencila.cloud/github/connect/site'

// =============================================================================
// Auth Status Cache
// =============================================================================

/**
 * Cache entry for auth status
 */
interface AuthCacheEntry {
  /** The promise that resolves to auth status */
  promise: Promise<SiteAuthStatusResponse | null>
  /** Timestamp when the cache entry was created */
  timestamp: number
}

/** Cache TTL in milliseconds (5 minutes) */
const AUTH_CACHE_TTL = 5 * 60 * 1000

/** Module-level cache keyed by API base URL */
const authCache = new Map<string, AuthCacheEntry>()

/**
 * Get cached auth status or fetch it.
 *
 * Multiple action components on the same page will share the same cached
 * promise, avoiding duplicate requests.
 *
 * @param apiBase - The API base URL (empty string for same-origin)
 * @param endpoint - The auth endpoint path
 * @param getHeaders - Function to get auth headers
 * @param forceRefresh - If true, bypass cache and fetch fresh data
 */
export async function getCachedAuthStatus(
  apiBase: string,
  endpoint: string,
  getHeaders: () => Promise<Record<string, string>>,
  forceRefresh: boolean = false
): Promise<SiteAuthStatusResponse | null> {
  const cacheKey = apiBase + endpoint
  const now = Date.now()

  // Check if we have a valid cached entry
  const cached = authCache.get(cacheKey)
  if (cached && !forceRefresh) {
    const age = now - cached.timestamp
    if (age < AUTH_CACHE_TTL) {
      return cached.promise
    }
  }

  // Create a new fetch promise
  const fetchPromise = fetchAuthStatus(apiBase, endpoint, getHeaders)

  // Store in cache immediately (so concurrent calls share the same promise)
  authCache.set(cacheKey, {
    promise: fetchPromise,
    timestamp: now,
  })

  // Remove from cache if fetch fails (so retries don't wait for TTL)
  fetchPromise.then((result) => {
    if (result === null) {
      authCache.delete(cacheKey)
    }
  })

  return fetchPromise
}

/**
 * Invalidate the auth cache for a specific API base, or all caches.
 * Call this after sign-in/sign-out to force a fresh fetch.
 */
export function invalidateAuthCache(apiBase?: string): void {
  if (apiBase !== undefined) {
    // Remove entries that start with this API base
    for (const key of authCache.keys()) {
      if (key.startsWith(apiBase)) {
        authCache.delete(key)
      }
    }
  } else {
    // Clear all cache entries
    authCache.clear()
  }
}

/**
 * Internal function to fetch auth status
 */
async function fetchAuthStatus(
  apiBase: string,
  endpoint: string,
  getHeaders: () => Promise<Record<string, string>>
): Promise<SiteAuthStatusResponse | null> {
  try {
    const headers = await getHeaders()

    const response = await fetch(apiBase + endpoint, {
      method: 'GET',
      headers,
      credentials: isLocalhost() ? 'include' : 'same-origin',
    })

    if (response.ok) {
      return await response.json()
    }

    console.error('[SiteAction] Failed to fetch auth status:', response.status)
    return null
  } catch (e) {
    console.error('[SiteAction] Auth status fetch failed:', e)
    return null
  }
}

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
