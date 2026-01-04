/**
 * Types for client-side navigation (glide)
 */

/**
 * A cached page entry
 *
 * Contains the extracted content and metadata from a fetched page,
 * stored for instant navigation on back/forward or repeated visits.
 */
export interface CacheEntry {
  /** Page title from <title> element */
  title: string

  /** Main content HTML from #main-content */
  mainHTML: string

  /** Meta description content (optional) */
  metaDescription?: string

  /** Canonical URL (optional) */
  canonical?: string

  /** Timestamp when the entry was cached */
  timestamp: number
}

/**
 * Navigation state stored in history
 *
 * Tracks scroll position for restoration on popstate events.
 */
export interface NavState {
  /** The URL that was navigated to */
  url: string

  /** Horizontal scroll position */
  scrollX: number

  /** Vertical scroll position */
  scrollY: number
}

/**
 * Configuration for client-side navigation
 *
 * Read from data attributes on <body> element and can be
 * overridden per-link with data attributes.
 */
export interface NavConfig {
  /** Whether client-side navigation is enabled */
  enabled: boolean

  /** Maximum prefetches per session (0 to disable prefetching) */
  prefetchLimit: number

  /** Maximum number of pages to cache (0 to disable caching) */
  cacheSize: number

  /** CSS selector for the main content element */
  contentSelector: string

  /** Maximum heading depth to include in TOC generation */
  tocMaxDepth: number
}

/** Default prefetch limit per session */
export const DEFAULT_PREFETCH_LIMIT = 20

/** Default cache size (number of pages) */
export const DEFAULT_CACHE_SIZE = 10

/**
 * Default configuration values
 */
export const DEFAULT_CONFIG: NavConfig = {
  enabled: true,
  prefetchLimit: DEFAULT_PREFETCH_LIMIT,
  cacheSize: DEFAULT_CACHE_SIZE,
  contentSelector: '#main-content',
  tocMaxDepth: 3,
}

/**
 * Navigation trigger source
 *
 * Indicates how the navigation was initiated.
 */
export type NavTrigger = 'click' | 'keyboard' | 'popstate' | 'programmatic'

/**
 * Event detail for glide navigation events
 */
export interface GlideEventDetail {
  /** The URL being navigated to */
  url: string

  /** How the navigation was triggered */
  trigger: NavTrigger

  /** Whether the navigation was from cache */
  fromCache?: boolean
}

/**
 * Event detail for glide error events
 */
export interface GlideErrorEventDetail extends GlideEventDetail {
  /** The error that occurred */
  error: Error
}

/**
 * Event detail for prefetch start events
 */
export interface PrefetchStartEventDetail {
  /** The URL being prefetched */
  url: string
}

/**
 * Event detail for prefetch end events
 */
export interface PrefetchEndEventDetail {
  /** The URL that was prefetched */
  url: string

  /** Whether the prefetch completed successfully */
  success: boolean
}
