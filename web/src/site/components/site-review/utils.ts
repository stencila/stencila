/**
 * Constants for site review components
 *
 * Note: Shared utility functions (isLocalhost, getPathname, isStencilaHostedSite, isDevMode)
 * have been moved to ../site-action/utils.ts
 */

// Storage keys
export const STORAGE_KEY_ITEMS = 'stencila-site-review-items'
export const STORAGE_KEY_SOURCE = 'stencila-site-review-source'

// API endpoint paths (relative, will be prefixed with apiBase)
export const REVIEW_SUBMIT_PATH = '/__stencila/reviews'
