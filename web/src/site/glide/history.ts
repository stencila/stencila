/**
 * History state and scroll position management
 *
 * Tracks scroll positions per URL for restoration on back/forward
 * navigation, and manages history.pushState/replaceState calls.
 */

import { scrollToId } from './scroll'
import type { NavState } from './types'

/** Map of URLs to their scroll positions */
const scrollPositions = new Map<string, { x: number; y: number }>()

/**
 * Save the current scroll position for a URL
 *
 * Called before navigating away from a page to preserve position
 * for back/forward navigation.
 */
export function saveScrollPosition(url: string): void {
  scrollPositions.set(url, {
    x: window.scrollX,
    y: window.scrollY,
  })
}

/**
 * Get the saved scroll position for a URL
 *
 * @returns Scroll position, or null if not saved
 */
export function getScrollPosition(url: string): { x: number; y: number } | null {
  return scrollPositions.get(url) ?? null
}

/**
 * Restore scroll position for a URL
 *
 * If no position is saved, scrolls to top (or to hash target if present).
 */
export function restoreScrollPosition(url: string): void {
  const saved = scrollPositions.get(url)

  if (saved) {
    window.scrollTo(saved.x, saved.y)
    return
  }

  // Check for hash target
  const hash = new URL(url, window.location.origin).hash
  if (hash && scrollToId(hash.slice(1))) {
    return
  }

  // Default: scroll to top
  window.scrollTo(0, 0)
}

/**
 * Push a new history entry for navigation
 *
 * Saves current scroll position before pushing new state.
 */
export function pushNavState(url: string): void {
  // Save scroll position for current page before navigating
  saveScrollPosition(window.location.href)

  const state: NavState = {
    url,
    scrollX: 0,
    scrollY: 0,
  }

  history.pushState(state, '', url)
}

/**
 * Replace current history entry
 *
 * Used for updating state without adding to history stack.
 */
export function replaceNavState(url: string): void {
  const state: NavState = {
    url,
    scrollX: window.scrollX,
    scrollY: window.scrollY,
  }

  history.replaceState(state, '', url)
}

/**
 * Check if a history state is a navigation state
 */
export function isNavState(state: unknown): state is NavState {
  return (
    typeof state === 'object' &&
    state !== null &&
    'url' in state &&
    typeof (state as NavState).url === 'string'
  )
}

/**
 * Clear all saved scroll positions
 *
 * Useful when the navigation module is reset or disabled.
 */
export function clearScrollPositions(): void {
  scrollPositions.clear()
}
