/**
 * Predictive prefetch manager for client-side navigation
 *
 * Listens for hover/focus on links and prefetches pages after a short
 * dwell time, storing results in the shared cache for instant navigation.
 */

import { getPageCache } from './cache'
import { dispatch, PrefetchEvents } from './events'
import { isEligibleForPrefetch, normalizeUrl } from './links'
import { parseHTML } from './parser'
import type { NavConfig } from './types'

/** Dwell time in ms before starting prefetch */
const DWELL_TIME = 100

/** Maximum concurrent prefetch requests */
const MAX_CONCURRENT = 2

/** NetworkInformation API type (not in standard lib) */
interface NetworkInformation {
  saveData?: boolean
  effectiveType?: '2g' | '3g' | '4g' | 'slow-2g'
}

/** Navigator with NetworkInformation API */
interface NavigatorWithConnection extends Navigator {
  connection?: NetworkInformation
}

/** URLs that have been prefetched this session */
const prefetchedUrls = new Set<string>()

/** Currently active prefetch count */
let activePrefetches = 0

/** Pending prefetch timeouts (URL -> AbortController) */
const pendingPrefetches = new Map<string, AbortController>()

/** Current configuration */
let config: NavConfig | null = null

/** Session prefetch count */
let sessionPrefetchCount = 0

/**
 * Check if prefetching should be skipped due to network conditions
 */
function shouldSkipForNetwork(): boolean {
  const nav = navigator as NavigatorWithConnection
  const connection = nav.connection

  if (!connection) {
    return false
  }

  // Skip if save-data is enabled
  if (connection.saveData) {
    return true
  }

  // Skip on slow connections
  const effectiveType = connection.effectiveType
  if (effectiveType === '2g' || effectiveType === 'slow-2g') {
    return true
  }

  return false
}

/**
 * Check if a URL can be prefetched
 */
function canPrefetch(url: string): boolean {
  // Prefetch requires both prefetching and caching to be enabled
  if (!config || config.prefetchLimit === 0 || config.cacheSize === 0) {
    return false
  }

  // Check session limit
  if (sessionPrefetchCount >= config.prefetchLimit) {
    return false
  }

  // Check concurrent limit
  if (activePrefetches >= MAX_CONCURRENT) {
    return false
  }

  // Skip if already prefetched
  const normalized = normalizeUrl(url)
  if (prefetchedUrls.has(normalized)) {
    return false
  }

  // Skip if already in cache
  const cache = getPageCache()
  if (cache.has(normalized)) {
    return false
  }

  // Skip if already pending
  if (pendingPrefetches.has(normalized)) {
    return false
  }

  // Check network conditions
  if (shouldSkipForNetwork()) {
    return false
  }

  return true
}

/**
 * Execute a prefetch request
 */
async function executePrefetch(url: string): Promise<void> {
  const normalized = normalizeUrl(url)

  // Double-check eligibility
  if (!canPrefetch(url)) {
    return
  }

  // Mark as attempted immediately - prevents retries on failure
  // and counts toward session limit regardless of outcome
  prefetchedUrls.add(normalized)
  sessionPrefetchCount++
  activePrefetches++

  // Dispatch start event
  dispatch(PrefetchEvents.START, { url: normalized })

  // Keep controller local - once fetch starts, it should complete
  // (only dwell timeouts are cancelable via pointerleave/focusout)
  const controller = new AbortController()
  let success = false

  try {
    const response = await fetch(url, {
      signal: controller.signal,
      headers: {
        'X-Stencila-Prefetch': '1',
      },
    })

    if (!response.ok) {
      return // Silently ignore errors
    }

    const html = await response.text()
    const entry = parseHTML(html, config?.contentSelector ?? '#main-content')

    if (entry) {
      // Store in cache
      const cache = getPageCache()
      cache.set(normalized, entry)

      success = true
    }
  } catch {
    // Silently ignore errors (including aborts)
  } finally {
    activePrefetches--

    // Always dispatch end event with success status
    dispatch(PrefetchEvents.END, { url: normalized, success })
  }
}

/**
 * Schedule a prefetch after dwell time
 */
function schedulePrefetch(url: string): void {
  const normalized = normalizeUrl(url)

  // Skip if already pending - prevents dwell reset when pointer moves
  // between child elements inside the same link
  if (pendingPrefetches.has(normalized)) {
    return
  }

  if (!canPrefetch(url)) {
    return
  }

  const controller = new AbortController()
  pendingPrefetches.set(normalized, controller)

  // Start prefetch after dwell time
  const timeoutId = window.setTimeout(() => {
    pendingPrefetches.delete(normalized)
    executePrefetch(url)
  }, DWELL_TIME)

  // Store timeout ID for cancellation (abuse signal for storage)
  controller.signal.addEventListener('abort', () => {
    window.clearTimeout(timeoutId)
  })
}

/**
 * Cancel a pending prefetch
 */
function cancelPrefetch(url: string): void {
  const normalized = normalizeUrl(url)
  const controller = pendingPrefetches.get(normalized)

  if (controller) {
    controller.abort()
    pendingPrefetches.delete(normalized)
  }
}

/**
 * Handle pointer enter on links
 */
function handlePointerEnter(event: PointerEvent): void {
  const target = event.target
  if (!(target instanceof Element)) {
    return
  }

  const link = target.closest('a')
  if (!(link instanceof HTMLAnchorElement)) {
    return
  }

  if (!isEligibleForPrefetch(link)) {
    return
  }

  schedulePrefetch(link.href)
}

/**
 * Handle pointer leave on links
 *
 * Uses relatedTarget to detect true exits - if we're moving to another
 * element within the same link, don't cancel the prefetch.
 */
function handlePointerLeave(event: PointerEvent): void {
  const target = event.target
  if (!(target instanceof Element)) {
    return
  }

  const link = target.closest('a')
  if (!(link instanceof HTMLAnchorElement)) {
    return
  }

  // Check if we're moving to another element within the same link
  const relatedTarget = event.relatedTarget
  if (relatedTarget instanceof Element && link.contains(relatedTarget)) {
    return // Still inside the link, don't cancel
  }

  cancelPrefetch(link.href)
}

/**
 * Handle focus on links
 */
function handleFocusIn(event: FocusEvent): void {
  const target = event.target
  if (!(target instanceof Element)) {
    return
  }

  const link = target.closest('a')
  if (!(link instanceof HTMLAnchorElement)) {
    return
  }

  if (!isEligibleForPrefetch(link)) {
    return
  }

  schedulePrefetch(link.href)
}

/**
 * Handle focus out on links
 */
function handleFocusOut(event: FocusEvent): void {
  const target = event.target
  if (!(target instanceof Element)) {
    return
  }

  const link = target.closest('a')
  if (!(link instanceof HTMLAnchorElement)) {
    return
  }

  cancelPrefetch(link.href)
}

/** Cleanup function */
let cleanup: (() => void) | null = null

/**
 * Initialize the prefetch manager
 *
 * @param navConfig - Navigation configuration
 * @returns Cleanup function to remove listeners
 */
export function initPrefetch(navConfig: NavConfig): () => void {
  // Clean up any previous initialization
  if (cleanup) {
    cleanup()
  }

  config = navConfig

  // Skip if prefetching is disabled
  if (config.prefetchLimit === 0) {
    return () => {}
  }

  // Set up event listeners (use capture for better interception)
  document.addEventListener('pointerenter', handlePointerEnter, true)
  document.addEventListener('pointerleave', handlePointerLeave, true)
  document.addEventListener('focusin', handleFocusIn, true)
  document.addEventListener('focusout', handleFocusOut, true)

  cleanup = () => {
    document.removeEventListener('pointerenter', handlePointerEnter, true)
    document.removeEventListener('pointerleave', handlePointerLeave, true)
    document.removeEventListener('focusin', handleFocusIn, true)
    document.removeEventListener('focusout', handleFocusOut, true)

    // Cancel all pending prefetches
    for (const controller of pendingPrefetches.values()) {
      controller.abort()
    }
    pendingPrefetches.clear()

    config = null
    cleanup = null
  }

  return cleanup
}

/**
 * Get the current prefetch stats (for debugging)
 */
export function getPrefetchStats(): {
  sessionCount: number
  activePrefetches: number
  prefetchedUrls: number
} {
  return {
    sessionCount: sessionPrefetchCount,
    activePrefetches,
    prefetchedUrls: prefetchedUrls.size,
  }
}
