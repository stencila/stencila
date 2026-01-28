/**
 * Access control utilities for site route restrictions
 *
 * This module provides utilities for:
 * - Fetching and caching _access.json route configuration
 * - Determining route access levels
 * - Checking if a user can access a route
 */

import type {
  AccessLevel,
  RouteAccessConfig,
} from '../components/site-action/types'

// =============================================================================
// Access Level Hierarchy
// =============================================================================

/**
 * Access levels in hierarchical order (lowest to highest)
 */
const ACCESS_LEVELS: AccessLevel[] = ['public', 'subscriber', 'password', 'team']

/**
 * Get the numeric index of an access level for comparison
 */
function getAccessLevelIndex(level: AccessLevel): number {
  return ACCESS_LEVELS.indexOf(level)
}

// =============================================================================
// Access Config Cache
// =============================================================================

/** Cached access config (null means not yet fetched, undefined means no restrictions) */
let accessConfigCache: RouteAccessConfig | null | undefined = null

/** Promise for in-flight fetch to prevent duplicate requests */
let fetchPromise: Promise<RouteAccessConfig | undefined> | null = null

/**
 * Fetch and cache the _access.json configuration
 *
 * Returns undefined if no access restrictions are configured (404 response).
 * Caches the result to avoid repeated fetches.
 */
export async function getAccessConfig(): Promise<RouteAccessConfig | undefined> {
  // Return cached value if available
  if (accessConfigCache !== null) {
    return accessConfigCache
  }

  // If fetch is in progress, wait for it
  if (fetchPromise) {
    return fetchPromise
  }

  // Start fetch
  fetchPromise = (async () => {
    try {
      const response = await fetch('/_access.json')

      if (response.status === 404) {
        // No access restrictions configured
        accessConfigCache = undefined
        return undefined
      }

      if (!response.ok) {
        console.warn(`Failed to fetch _access.json: ${response.status}`)
        accessConfigCache = undefined
        return undefined
      }

      const config: RouteAccessConfig = await response.json()
      accessConfigCache = config
      return config
    } catch (error) {
      console.warn('Error fetching _access.json:', error)
      accessConfigCache = undefined
      return undefined
    } finally {
      fetchPromise = null
    }
  })()

  return fetchPromise
}

/**
 * Clear the access config cache
 *
 * Useful for testing or when the config may have changed.
 */
export function clearAccessConfigCache(): void {
  accessConfigCache = null
  fetchPromise = null
}

// =============================================================================
// Route Access Level
// =============================================================================

/**
 * Get the required access level for a route path
 *
 * Uses longest prefix match to find the most specific route configuration.
 * Falls back to the default access level if no route matches.
 *
 * @param path - The route path (e.g., "/data/report/")
 * @param config - The access configuration
 * @returns The required access level for the route
 */
export function getRouteAccessLevel(
  path: string,
  config: RouteAccessConfig
): AccessLevel {
  // Normalize path to ensure it starts with /
  const normalizedPath = path.startsWith('/') ? path : `/${path}`

  // Find longest matching route prefix
  let bestMatch: { route: string; level: AccessLevel } | null = null

  for (const [route, level] of Object.entries(config.routes)) {
    // Routes end with '/' so prefix matching naturally works at path boundaries
    // e.g., "/data/" matches "/data/" and "/data/file.csv" but not "/database/"
    if (normalizedPath.startsWith(route)) {
      if (!bestMatch || route.length > bestMatch.route.length) {
        bestMatch = { route, level }
      }
    }
  }

  return bestMatch ? bestMatch.level : config.default
}

// =============================================================================
// Access Checking
// =============================================================================

/**
 * Check if a user with the given access level can access a route
 *
 * @param userLevel - The user's maximum access level
 * @param routePath - The route path to check
 * @param config - The access configuration
 * @returns True if the user can access the route
 */
export function canAccessRoute(
  userLevel: AccessLevel,
  routePath: string,
  config: RouteAccessConfig
): boolean {
  const requiredLevel = getRouteAccessLevel(routePath, config)
  return canAccess(userLevel, requiredLevel)
}

/**
 * Check if a user access level meets or exceeds a required level
 *
 * @param userLevel - The user's access level
 * @param requiredLevel - The required access level
 * @returns True if userLevel >= requiredLevel in the hierarchy
 */
export function canAccess(
  userLevel: AccessLevel,
  requiredLevel: AccessLevel
): boolean {
  return getAccessLevelIndex(userLevel) >= getAccessLevelIndex(requiredLevel)
}

/**
 * Get all access levels a user can access (their level and below)
 *
 * @param userLevel - The user's access level
 * @returns Array of accessible levels, from public up to userLevel
 */
export function getAccessibleLevels(userLevel: AccessLevel): AccessLevel[] {
  const index = getAccessLevelIndex(userLevel)
  return ACCESS_LEVELS.slice(0, index + 1)
}

// =============================================================================
// Utilities
// =============================================================================

/**
 * Check if any access restrictions are configured
 *
 * @param config - The access configuration (or undefined if none)
 * @returns True if there are non-public restrictions
 */
export function hasAccessRestrictions(
  config: RouteAccessConfig | undefined
): config is RouteAccessConfig {
  if (!config) {
    return false
  }

  // Check if default is non-public or any routes are non-public
  if (config.default !== 'public') {
    return true
  }

  return Object.values(config.routes).some((level) => level !== 'public')
}

// =============================================================================
// Access Badge Rendering
// =============================================================================

/**
 * Get the tooltip title for an access level badge
 */
export function getAccessBadgeTitle(level: AccessLevel): string {
  switch (level) {
    case 'subscriber':
      return 'Subscribers only'
    case 'password':
      return 'Password required'
    case 'team':
      return 'Team members only'
    default:
      return ''
  }
}

/**
 * Get the icon class for an access level badge
 */
export function getAccessBadgeIcon(level: AccessLevel): string {
  switch (level) {
    case 'subscriber':
      return 'i-lucide:star'
    case 'password':
      return 'i-lucide:lock'
    case 'team':
      return 'i-lucide:users'
    default:
      return ''
  }
}

/**
 * Create an access badge element for a nav item
 *
 * @param level - The required access level
 * @returns The badge element, or null if level is 'public'
 */
export function createAccessBadge(level: AccessLevel): HTMLSpanElement | null {
  if (level === 'public') {
    return null
  }

  const badge = document.createElement('span')
  badge.className = 'nav-access-badge'
  badge.dataset.level = level
  badge.title = getAccessBadgeTitle(level)

  const icon = document.createElement('span')
  icon.className = `icon ${getAccessBadgeIcon(level)}`
  badge.appendChild(icon)

  return badge
}

/**
 * Add an access badge to a nav item element
 *
 * Finds the appropriate link/label element and appends the badge.
 * If no element matches the selector, appends directly to the item.
 * Does nothing if badge already exists or level is 'public'.
 *
 * @param item - The nav item element (should have data-access attribute)
 * @param level - The required access level
 * @param linkSelector - CSS selector for the element to append badge to
 * @returns True if badge was added
 */
export function addAccessBadgeToItem(
  item: HTMLElement,
  level: AccessLevel,
  linkSelector: string = 'a, .group-link, .label'
): boolean {
  // Don't add badge for public or if one already exists
  if (level === 'public' || item.querySelector('.nav-access-badge')) {
    return false
  }

  // Find the link or label element to append badge to
  // Fall back to item itself if no match (e.g., group heading without link)
  const target = item.querySelector(linkSelector) ?? item

  const badge = createAccessBadge(level)
  if (badge) {
    target.appendChild(badge)
    return true
  }

  return false
}
