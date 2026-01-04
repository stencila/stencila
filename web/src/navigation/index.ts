/**
 * Client-side navigation module
 *
 * Provides instant page-to-page navigation without full reloads,
 * using the View Transitions API when available.
 */

// Re-export types
export type {
  CacheEntry,
  NavState,
  NavConfig,
  NavTrigger,
  GlideEventDetail,
  GlideErrorEventDetail,
  PrefetchEventDetail,
} from './types'

export { DEFAULT_CONFIG, DEFAULT_PREFETCH_LIMIT, DEFAULT_CACHE_SIZE } from './types'

// Re-export events
export { GlideEvents, PrefetchEvents, GLIDE_REQUEST, dispatch } from './events'

// Re-export cache
export { PageCache, getPageCache, initPageCache } from './cache'
