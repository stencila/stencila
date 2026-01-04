/**
 * Main navigation controller for client-side page transitions
 *
 * Intercepts link clicks, fetches pages, and swaps content using
 * View Transitions API when available.
 */

import type { StencilaNavTree } from '../layout/nav-tree'
import type { StencilaTocTree } from '../layout/toc-tree'

import { getPageCache, initPageCache } from './cache'
import { dispatch, GLIDE_REQUEST, GlideEvents } from './events'
import {
  pushNavState,
  restoreScrollPosition,
  saveScrollPosition,
} from './history'
import { parseHTML } from './parser'
import { scrollToId } from './scroll'
import { generateTocFromHeadings } from './toc'
import { DEFAULT_CONFIG } from './types'
import type { GlideEventDetail, NavConfig, NavTrigger } from './types'

/** Current configuration */
let config: NavConfig = { ...DEFAULT_CONFIG }

/**
 * Normalize a URL for cache key (excludes hash fragment)
 *
 * This ensures that different hash targets on the same page share a cache entry.
 */
function normalizeUrlForCache(url: string): string {
  const parsed = new URL(url, window.location.origin)
  return parsed.origin + parsed.pathname + parsed.search
}

/** Whether navigation is currently in progress */
let isNavigating = false

/** Last normalized URL (without hash) to detect hash-only popstate */
let lastNormalizedUrl = ''

/** Last full URL (with hash) for scroll position tracking */
let lastFullUrl = ''

/**
 * Check if a link is eligible for client-side navigation
 */
function isEligibleLink(link: HTMLAnchorElement): boolean {
  // Must have href
  if (!link.href) {
    return false
  }

  // Must be same origin
  const url = new URL(link.href, window.location.origin)
  if (url.origin !== window.location.origin) {
    return false
  }

  // Skip if link has data-stencila-glide="off"
  if (link.dataset.stencilaGlide === 'off') {
    return false
  }

  // Skip download links
  if (link.hasAttribute('download')) {
    return false
  }

  // Skip links with target other than _self
  const target = link.getAttribute('target')
  if (target && target !== '_self') {
    return false
  }

  // Skip mailto, tel, javascript links
  const protocol = url.protocol
  if (protocol !== 'http:' && protocol !== 'https:') {
    return false
  }

  return true
}

/**
 * Check if a click event should trigger navigation
 */
function isEligibleClick(event: MouseEvent): boolean {
  // Only left clicks
  if (event.button !== 0) {
    return false
  }

  // No modifier keys (allow normal browser behavior)
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) {
    return false
  }

  return true
}

/**
 * Perform the DOM swap with optional View Transition
 */
async function swapContent(
  mainHTML: string,
  title: string,
  contentSelector: string
): Promise<void> {
  const mainElement = document.querySelector(contentSelector)
  if (!mainElement) {
    throw new Error(`Content element not found: ${contentSelector}`)
  }

  const doSwap = () => {
    // Update title
    document.title = title

    // Swap main content
    mainElement.innerHTML = mainHTML

    // Focus management: focus the first h1 for screen readers
    const h1 = mainElement.querySelector('h1')
    if (h1) {
      h1.setAttribute('tabindex', '-1')
      h1.focus({ preventScroll: true })
    }
  }

  // Use View Transitions API if available and motion is allowed
  const prefersReducedMotion = window.matchMedia(
    '(prefers-reduced-motion: reduce)'
  ).matches

  if ('startViewTransition' in document && !prefersReducedMotion) {
    await (document as Document & { startViewTransition: (cb: () => void) => { finished: Promise<void> } })
      .startViewTransition(doSwap)
      .finished
  } else {
    doSwap()
  }
}

/**
 * Rehydrate components after content swap
 *
 * Updates the TOC and nav tree to reflect the new page content.
 * Also exposed as window.__stencilaRehydrate for external callers.
 */
export function rehydrateComponents(url: string, mainElement: Element): void {
  // Update TOC with new headings
  const tocTree = document.querySelector('stencila-toc-tree') as StencilaTocTree | null
  if (tocTree && mainElement instanceof HTMLElement) {
    const tocHtml = generateTocFromHeadings(mainElement, config.tocMaxDepth)
    const tocContainer = tocTree.querySelector('ul[role="tree"]')
    if (tocContainer) {
      // Replace the TOC content
      tocContainer.outerHTML = tocHtml || '<ul role="tree" class="toc-list"></ul>'
    } else if (tocHtml) {
      // No existing TOC, insert new one
      tocTree.innerHTML = tocHtml
    }
    // Reinitialize the TOC tree component
    tocTree.reinitialize()
  }

  // Update nav tree active link
  const navTree = document.querySelector('stencila-nav-tree') as StencilaNavTree | null
  if (navTree) {
    navTree.updateActiveLink(url)
  }
}

/** Options for navigate() */
interface NavigateOptions {
  /** Skip pushing to history (used for popstate) */
  skipPush?: boolean
}

/**
 * Navigate to a URL using client-side navigation
 */
export async function navigate(
  url: string,
  trigger: NavTrigger = 'programmatic',
  options: NavigateOptions = {}
): Promise<boolean> {
  // Prevent concurrent navigations
  if (isNavigating) {
    return false
  }

  // Skip if navigating to current page (but allow hash changes)
  const targetUrl = new URL(url, window.location.origin)
  const currentUrl = new URL(window.location.href)

  if (
    targetUrl.pathname === currentUrl.pathname &&
    targetUrl.search === currentUrl.search &&
    targetUrl.hash === currentUrl.hash
  ) {
    return false
  }

  // Handle same-page hash navigation
  if (
    targetUrl.pathname === currentUrl.pathname &&
    targetUrl.search === currentUrl.search &&
    targetUrl.hash
  ) {
    if (scrollToId(targetUrl.hash.slice(1), 'smooth')) {
      if (!options.skipPush) {
        saveScrollPosition(window.location.href)
        history.pushState(null, '', url)
      }
      return true
    }
  }

  isNavigating = true

  const detail: GlideEventDetail = { url, trigger }

  try {
    // Dispatch start event
    dispatch(GlideEvents.START, detail)

    // Check cache first (normalize URL to exclude hash)
    const cache = getPageCache()
    const cacheKey = normalizeUrlForCache(url)
    let entry = config.cacheSize > 0 ? cache.get(cacheKey) : undefined
    let fromCache = false

    if (entry) {
      fromCache = true
    } else {
      // Fetch the page
      const response = await fetch(url, {
        headers: {
          'X-Stencila-Glide': '1',
        },
      })

      if (!response.ok) {
        throw new Error(`Failed to fetch: ${response.status}`)
      }

      const html = await response.text()
      entry = parseHTML(html, config.contentSelector)

      if (!entry) {
        throw new Error('Failed to parse page content')
      }

      // Cache the result
      if (config.cacheSize > 0) {
        cache.set(cacheKey, entry)
      }
    }

    // Dispatch before-swap event (cancelable)
    const detailWithCache = { ...detail, fromCache }
    if (!dispatch(GlideEvents.BEFORE_SWAP, detailWithCache, true)) {
      isNavigating = false
      return false
    }

    // Update history before swap (skip for popstate to avoid corrupting history)
    if (!options.skipPush) {
      pushNavState(url)
    }

    // Perform the swap
    await swapContent(entry.mainHTML, entry.title, config.contentSelector)

    // Rehydrate components (TOC, nav tree)
    const mainElement = document.querySelector(config.contentSelector)
    if (mainElement) {
      rehydrateComponents(url, mainElement)
    }

    // Dispatch after-swap event
    dispatch(GlideEvents.AFTER_SWAP, detailWithCache)

    // Track current page for hash-only popstate detection
    lastNormalizedUrl = normalizeUrlForCache(url)
    lastFullUrl = url

    // Handle scroll position
    restoreScrollPosition(url)

    // Dispatch end event
    dispatch(GlideEvents.END, detailWithCache)

    isNavigating = false
    return true
  } catch (error) {
    isNavigating = false

    // Dispatch error event
    dispatch(GlideEvents.ERROR, {
      ...detail,
      error: error instanceof Error ? error : new Error(String(error)),
    })

    // Fallback to full page load
    window.location.href = url
    return false
  }
}

/**
 * Handle click events on links
 */
function handleClick(event: MouseEvent): void {
  if (!config.enabled) {
    return
  }

  // Respect other handlers that cancelled this event
  if (event.defaultPrevented) {
    return
  }

  if (!isEligibleClick(event)) {
    return
  }

  // Find the closest anchor element
  const target = event.target
  if (!(target instanceof Element)) {
    return
  }
  const link = target.closest('a')
  if (!(link instanceof HTMLAnchorElement)) {
    return
  }

  if (!isEligibleLink(link)) {
    return
  }

  // Check if this would actually navigate before preventing default
  const targetUrl = new URL(link.href, window.location.origin)
  const currentUrl = new URL(window.location.href)
  const isSameUrl =
    targetUrl.pathname === currentUrl.pathname &&
    targetUrl.search === currentUrl.search &&
    targetUrl.hash === currentUrl.hash

  // For same-URL clicks, let browser handle (reload behavior)
  if (isSameUrl) {
    return
  }

  event.preventDefault()
  navigate(link.href, 'click')
}

/**
 * Handle popstate events (back/forward navigation)
 */
function handlePopstate(_event: PopStateEvent): void {
  if (!config.enabled) {
    return
  }

  // If navigation is in progress, force reload to avoid inconsistent state
  if (isNavigating) {
    window.location.reload()
    return
  }

  const url = window.location.href
  const cacheKey = normalizeUrlForCache(url)

  // Save scroll position for the page we're leaving before any swap
  if (lastFullUrl) {
    saveScrollPosition(lastFullUrl)
  }

  // Hash-only change on same page: just scroll, don't swap content
  if (cacheKey === lastNormalizedUrl) {
    lastFullUrl = url
    restoreScrollPosition(url)
    return
  }

  // Check if we have cached content
  const cache = getPageCache()
  const entry = config.cacheSize > 0 ? cache.get(cacheKey) : undefined

  if (entry) {
    // Use cached content for instant back/forward
    const detail: GlideEventDetail = { url, trigger: 'popstate', fromCache: true }
    dispatch(GlideEvents.START, detail)

    if (dispatch(GlideEvents.BEFORE_SWAP, detail, true)) {
      swapContent(entry.mainHTML, entry.title, config.contentSelector)
        .then(() => {
          // Rehydrate components (TOC, nav tree)
          const mainElement = document.querySelector(config.contentSelector)
          if (mainElement) {
            rehydrateComponents(url, mainElement)
          }

          dispatch(GlideEvents.AFTER_SWAP, detail)
          lastNormalizedUrl = cacheKey
          lastFullUrl = url
          restoreScrollPosition(url)
          dispatch(GlideEvents.END, detail)
        })
        .catch(() => {
          // Fallback on error
          window.location.reload()
        })
    }
  } else {
    // No cache, fetch without pushing to history
    navigate(url, 'popstate', { skipPush: true })
  }
}

/**
 * Handle programmatic navigation requests
 */
function handleGlideRequest(event: CustomEvent<{ url: string; trigger: string }>): void {
  if (!config.enabled) {
    return
  }

  const { url, trigger } = event.detail
  navigate(url, (trigger as NavTrigger) || 'programmatic')
}

/**
 * Parse a non-negative integer from a data attribute
 */
function parseNonNegativeInt(value: string | undefined): number | undefined {
  if (value === undefined) return undefined
  const parsed = parseInt(value, 10)
  return !isNaN(parsed) && parsed >= 0 ? parsed : undefined
}

/**
 * Get a data attribute value, checking documentElement first then body
 */
function getDataAttr(name: string): string | undefined {
  return document.documentElement.dataset[name] ?? document.body.dataset[name]
}

/**
 * Read configuration from data attributes
 */
function readConfig(): NavConfig {
  return {
    ...DEFAULT_CONFIG,
    enabled: getDataAttr('stencilaGlide') === 'off' ? false
           : getDataAttr('stencilaGlide') === 'on' ? true
           : DEFAULT_CONFIG.enabled,
    prefetchLimit: parseNonNegativeInt(getDataAttr('stencilaPrefetch')) ?? DEFAULT_CONFIG.prefetchLimit,
    cacheSize: parseNonNegativeInt(getDataAttr('stencilaCache')) ?? DEFAULT_CONFIG.cacheSize,
  }
}

/** Cleanup function returned by init */
let cleanup: (() => void) | null = null

/**
 * Initialize the navigation controller
 *
 * Sets up event listeners and reads configuration from data attributes.
 * Returns a cleanup function to remove listeners.
 */
export function initNavigation(): () => void {
  // Clean up any previous initialization
  if (cleanup) {
    cleanup()
  }

  // Read config from data attributes
  config = readConfig()

  if (!config.enabled) {
    return () => {}
  }

  // Take control of scroll restoration from browser
  history.scrollRestoration = 'manual'

  // Track current page for hash-only popstate detection
  lastNormalizedUrl = normalizeUrlForCache(window.location.href)
  lastFullUrl = window.location.href

  // Initialize cache with configured size
  if (config.cacheSize > 0) {
    initPageCache(config.cacheSize)
  }

  // Cache the current page
  if (config.cacheSize > 0) {
    const mainElement = document.querySelector(config.contentSelector)
    if (mainElement) {
      getPageCache().set(normalizeUrlForCache(window.location.href), {
        title: document.title,
        mainHTML: mainElement.innerHTML,
        timestamp: Date.now(),
      })
    }
  }

  // Save initial scroll position
  saveScrollPosition(window.location.href)

  // Set up event listeners
  document.addEventListener('click', handleClick)
  window.addEventListener('popstate', handlePopstate)
  window.addEventListener(GLIDE_REQUEST, handleGlideRequest as EventListener)

  cleanup = () => {
    document.removeEventListener('click', handleClick)
    window.removeEventListener('popstate', handlePopstate)
    window.removeEventListener(GLIDE_REQUEST, handleGlideRequest as EventListener)
    history.scrollRestoration = 'auto'
    cleanup = null
  }

  return cleanup
}

/**
 * Get the current configuration
 */
export function getConfig(): Readonly<NavConfig> {
  return config
}
