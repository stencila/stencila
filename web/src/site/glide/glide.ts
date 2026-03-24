/**
 * Main navigation controller for client-side page transitions
 *
 * Intercepts link clicks, fetches pages, and swaps content using
 * View Transitions API when available.
 */

import type { StencilaNavTree } from '../components/nav-tree'
import type { StencilaTocTree } from '../components/toc-tree'

import { getPageCache, initPageCache } from './cache'
import { GLIDE_REQUEST, GlideEvents, dispatch } from './events'
import {
  pushNavState,
  restoreScrollPosition,
  saveScrollPosition,
} from './history'
import { isEligibleLink, normalizeUrl } from './links'
import { parseHTML } from './parser'
import { initPrefetch } from './prefetch'
import { scrollToId } from './scroll'
import { generateTocFromHeadings } from './toc'
import { DEFAULT_CONFIG } from './types'
import type { CacheEntry, GlideEventDetail, NavConfig, NavTrigger } from './types'

/** Current configuration */
let config: NavConfig = { ...DEFAULT_CONFIG }

/** Whether navigation is currently in progress */
let isNavigating = false

/** Last normalized URL (without hash) to detect hash-only popstate */
let lastNormalizedUrl = ''

/** Last full URL (with hash) for scroll position tracking */
let lastFullUrl = ''

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
 * Scroll target for content swap
 */
type ScrollTarget =
  | { type: 'top' }
  | { type: 'hash'; hash: string }
  | { type: 'restore'; url: string }

/**
 * Swap sidebar content, creating or removing sidebar elements as needed
 *
 * This handles the case where navigating from a page without sidebars
 * to a page with sidebars (or vice versa) requires creating/removing
 * the sidebar elements in the DOM.
 */
function swapSidebars(entry: CacheEntry, contentSelector: string): void {
  const layoutBody = document.querySelector('.layout-body')
  const layout = document.querySelector('stencila-layout')
  const mainElement = document.querySelector(contentSelector)

  if (!layoutBody || !layout || !mainElement) {
    return
  }

  // Handle left sidebar
  const existingLeft = document.querySelector('stencila-left-sidebar')
  if (entry.leftSidebarHTML !== undefined) {
    if (existingLeft) {
      // Update existing sidebar
      existingLeft.innerHTML = entry.leftSidebarHTML
    } else {
      // Create new sidebar and insert before .layout-main
      const newLeft = document.createElement('stencila-left-sidebar')
      newLeft.innerHTML = entry.leftSidebarHTML
      layoutBody.insertBefore(newLeft, mainElement)
    }
    layout.setAttribute('left-sidebar', '')
  } else if (existingLeft) {
    // Remove sidebar that's no longer needed
    existingLeft.remove()
    layout.removeAttribute('left-sidebar')
  }

  // Handle right sidebar
  const existingRight = document.querySelector('stencila-right-sidebar')
  if (entry.rightSidebarHTML !== undefined) {
    if (existingRight) {
      // Update existing sidebar
      existingRight.innerHTML = entry.rightSidebarHTML
    } else {
      // Create new sidebar and insert after .layout-main
      const newRight = document.createElement('stencila-right-sidebar')
      newRight.innerHTML = entry.rightSidebarHTML
      // Insert after .layout-main (before next sibling or at end)
      if (mainElement.nextSibling) {
        layoutBody.insertBefore(newRight, mainElement.nextSibling)
      } else {
        layoutBody.appendChild(newRight)
      }
    }
    layout.setAttribute('right-sidebar', '')
  } else if (existingRight) {
    // Remove sidebar that's no longer needed
    existingRight.remove()
    layout.removeAttribute('right-sidebar')
  }
}

/**
 * Update main content area attributes on the layout element
 *
 * These attributes control CSS custom property overrides for content
 * width, padding, and title visibility. They need to be synced
 * when navigating between pages with different layout configurations.
 */
function updateLayoutAttributes(entry: CacheEntry): void {
  const layout = document.querySelector('stencila-layout')
  if (!layout) {
    return
  }

  // Helper to set or remove an attribute
  const sync = (attr: string, value?: string) => {
    if (value) {
      layout.setAttribute(attr, value)
    } else {
      layout.removeAttribute(attr)
    }
  }

  sync('main-width', entry.mainWidth)
  sync('main-padding', entry.mainPadding)
  sync('main-title', entry.mainTitle)
}

/**
 * Perform the DOM swap with optional View Transition
 *
 * For 'restore' and 'top' scroll targets, scrolling happens during the
 * View Transition for smooth animated transitions. For 'hash' targets,
 * scrolling happens after the transition to ensure reliable positioning.
 */
async function swapContent(
  entry: CacheEntry,
  contentSelector: string,
  scrollTarget: ScrollTarget
): Promise<void> {
  const mainElement = document.querySelector(contentSelector)
  if (!mainElement) {
    throw new Error(`Content element not found: ${contentSelector}`)
  }

  const doSwap = () => {
    // Update title
    document.title = entry.title

    // Swap main content
    mainElement.innerHTML = entry.mainHTML

    // Swap sidebars (create/update/remove as needed)
    swapSidebars(entry, contentSelector)

    // Update content formatting attributes on layout element
    updateLayoutAttributes(entry)

    // Handle scroll for restore/top as part of the transition
    // Hash scrolling is done after transition for reliable positioning
    switch (scrollTarget.type) {
      case 'restore':
        restoreScrollPosition(scrollTarget.url)
        break
      case 'top':
        window.scrollTo(0, 0)
        break
    }

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

  // Scroll to hash target after transition completes
  // Use rAF to ensure View Transition has fully settled before scrolling
  if (scrollTarget.type === 'hash') {
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        if (!scrollToId(scrollTarget.hash)) {
          window.scrollTo(0, 0)
        }
        resolve()
      })
    })
  }
}

/**
 * Rehydrate components after content swap
 *
 * Updates the TOC and nav tree to reflect the new page content.
 * TOC updates use a crossfade to avoid visual jumping.
 */
export function rehydrateComponents(url: string, mainElement: Element): void {
  // Update TOC with new headings
  const rightSidebar = document.querySelector(
    'nav[slot="right-sidebar"]'
  ) as HTMLElement | null
  const tocTree = document.querySelector(
    'stencila-toc-tree'
  ) as StencilaTocTree | null

  if (mainElement instanceof HTMLElement && rightSidebar) {
    const tocHtml = generateTocFromHeadings(mainElement, config.tocMaxDepth)
    const hasHeadings = tocHtml.length > 0

    // Toggle data-empty attribute - CSS hides content but keeps layout space
    if (hasHeadings) {
      rightSidebar.removeAttribute('data-empty')
    } else {
      rightSidebar.setAttribute('data-empty', '')
    }

    if (tocTree) {
      // Crossfade the TOC to avoid visual jumping
      const fadeDuration = 100
      tocTree.style.opacity = '0'
      tocTree.style.transition = `opacity ${fadeDuration}ms ease-out`

      // Wait for fade out to complete before updating content
      setTimeout(() => {
        const tocContainer = tocTree.querySelector('ul[role="tree"]')
        if (tocContainer) {
          tocContainer.outerHTML = tocHtml
        } else {
          tocTree.innerHTML = tocHtml
        }
        tocTree.reinitialize()

        // Fade back in
        requestAnimationFrame(() => {
          tocTree.style.opacity = '1'
          // Clean up transition after it completes
          setTimeout(() => {
            tocTree.style.transition = ''
          }, fadeDuration)
        })
      }, fadeDuration)
    }
  }

  // Update nav tree active link
  const navTree = document.querySelector(
    'stencila-nav-tree'
  ) as StencilaNavTree | null
  if (navTree) {
    navTree.updateActiveLink(url)
  }
}

/**
 * Coerce an unknown thrown value into an Error instance
 */
function toError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error))
}

/**
 * Update the module-level page tracking state
 *
 * Called after every successful content swap so that hash-only popstate
 * detection and scroll-position bookkeeping stay consistent.
 */
function setCurrentPage(url: string): void {
  lastNormalizedUrl = normalizeUrl(url)
  lastFullUrl = url
}

/**
 * Fetch a page and parse it into a cache entry, optionally using
 * the glide cache for lookups and storage.
 *
 * When `invalidateCache` is true the existing cache entry (if any)
 * is deleted before the fetch so the caller always gets fresh content.
 */
async function fetchEntry(
  url: string,
  invalidateCache = false
): Promise<{ entry: CacheEntry; cacheKey: string; fromCache: boolean }> {
  const cache = getPageCache()
  const cacheKey = normalizeUrl(url)

  if (invalidateCache) {
    cache.delete(cacheKey)
  }

  // Try the cache first (unless we just invalidated it)
  const cached = !invalidateCache && config.cacheSize > 0
    ? cache.get(cacheKey)
    : undefined

  if (cached) {
    return { entry: cached, cacheKey, fromCache: true }
  }

  const response = await fetch(url, {
    headers: { 'X-Stencila-Glide': '1' },
  })

  if (!response.ok) {
    throw new Error(`Failed to fetch: ${response.status}`)
  }

  const html = await response.text()
  const entry = parseHTML(html, config.contentSelector)

  if (!entry) {
    throw new Error('Failed to parse page content')
  }

  if (config.cacheSize > 0) {
    cache.set(cacheKey, entry)
  }

  return { entry, cacheKey, fromCache: false }
}

/**
 * Swap page content, rehydrate UI components, and dispatch post-swap
 * lifecycle events.
 */
async function applyEntry(
  url: string,
  entry: CacheEntry,
  scrollTarget: ScrollTarget,
  detailWithCache: GlideEventDetail
): Promise<void> {
  await swapContent(entry, config.contentSelector, scrollTarget)

  const mainElement = document.querySelector(config.contentSelector)
  if (mainElement) {
    rehydrateComponents(url, mainElement)
  }

  dispatch(GlideEvents.AFTER_SWAP, detailWithCache)
  setCurrentPage(url)
  dispatch(GlideEvents.END, detailWithCache)
}

/**
 * Run `fn` while the `isNavigating` guard is held.
 *
 * The guard is always released — even if `fn` throws — unless
 * `fn` triggers a hard navigation (e.g. `window.location.href = …`),
 * in which case the page is going away anyway.
 */
async function withNavigationGuard<T>(fn: () => Promise<T>): Promise<T | false> {
  if (isNavigating) {
    return false
  }
  isNavigating = true
  try {
    return await fn()
  } finally {
    isNavigating = false
  }
}

/**
 * Reload the current page content without a full page refresh
 *
 * Re-fetches the current URL from the server, invalidates the glide cache
 * entry, and swaps content using View Transitions for a smooth update.
 * Preserves the current scroll position and dispatches glide lifecycle
 * events so that other code (loading indicators, analytics, etc.) can
 * observe the update.
 */
export async function reload(): Promise<boolean> {
  const url = window.location.href
  const detail: GlideEventDetail = { url, trigger: 'programmatic' }

  const result = await withNavigationGuard(async () => {
    dispatch(GlideEvents.START, detail)

    let entry: CacheEntry
    let fromCache: boolean
    try {
      ;({ entry, fromCache } = await fetchEntry(url, true))
    } catch (error) {
      dispatch(GlideEvents.ERROR, { ...detail, error: toError(error) })
      console.warn('Glide reload failed, falling back to full reload:', error)
      window.location.reload()
      return false
    }

    const detailWithCache = { ...detail, fromCache }
    if (!dispatch(GlideEvents.BEFORE_SWAP, detailWithCache, true)) {
      return false
    }

    saveScrollPosition(url)
    await applyEntry(url, entry, { type: 'restore', url }, detailWithCache)

    return true
  })

  return result === false ? false : result
}

/**
 * Reload all stylesheets without a full page refresh
 *
 * Appends a cache-busting query parameter to each local stylesheet's href,
 * causing the browser to re-fetch the CSS while keeping the DOM intact.
 * A single timestamp is used for all links so that co-dependent stylesheets
 * are fetched from the same build.
 */
export function reloadStyles(): void {
  const links = document.querySelectorAll<HTMLLinkElement>(
    'link[rel="stylesheet"]'
  )
  const cacheBust = String(Date.now())

  for (const link of links) {
    const href = link.getAttribute('href')
    if (!href) continue

    // Only reload same-origin stylesheets
    try {
      const url = new URL(href, window.location.origin)
      if (url.origin !== window.location.origin) continue

      url.searchParams.set('_t', cacheBust)
      link.setAttribute('href', url.href)
    } catch {
      console.debug('⚠️ Skipping malformed stylesheet URL:', href)
    }
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

  const detail: GlideEventDetail = { url, trigger }

  const result = await withNavigationGuard(async () => {
    dispatch(GlideEvents.START, detail)

    let entry: CacheEntry
    let fromCache: boolean
    try {
      ;({ entry, fromCache } = await fetchEntry(url))
    } catch (error) {
      dispatch(GlideEvents.ERROR, { ...detail, error: toError(error) })
      // Fallback to full page load
      window.location.href = url
      return false
    }

    // Dispatch before-swap event (cancelable)
    const detailWithCache = { ...detail, fromCache }
    if (!dispatch(GlideEvents.BEFORE_SWAP, detailWithCache, true)) {
      return false
    }

    // Update history before swap (skip for popstate to avoid corrupting history)
    if (!options.skipPush) {
      pushNavState(url)
    }

    // Determine scroll target: restore for popstate, otherwise top or hash
    const hash = new URL(url, window.location.origin).hash.slice(1)
    const scrollTarget: ScrollTarget =
      trigger === 'popstate'
        ? { type: 'restore', url }
        : hash
          ? { type: 'hash', hash }
          : { type: 'top' }

    await applyEntry(url, entry, scrollTarget, detailWithCache)

    return true
  })

  return result === false ? false : result
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
  const cacheKey = normalizeUrl(url)

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
    // Use cached content for instant back/forward, guarded to
    // prevent concurrent navigations during the async swap.
    const detail: GlideEventDetail = { url, trigger: 'popstate', fromCache: true }
    isNavigating = true

    dispatch(GlideEvents.START, detail)

    if (dispatch(GlideEvents.BEFORE_SWAP, detail, true)) {
      applyEntry(url, entry, { type: 'restore', url }, detail)
        .catch(() => {
          // Fallback on error
          window.location.reload()
        })
        .finally(() => {
          isNavigating = false
        })
    } else {
      isNavigating = false
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
export function initSiteGlide(): () => void {
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
  lastNormalizedUrl = normalizeUrl(window.location.href)
  lastFullUrl = window.location.href

  // Initialize cache with configured size
  if (config.cacheSize > 0) {
    initPageCache(config.cacheSize)
  }

  // Cache the current page (including sidebars)
  if (config.cacheSize > 0) {
    const mainElement = document.querySelector(config.contentSelector)
    if (mainElement) {
      const leftSidebar = document.querySelector('stencila-left-sidebar')
      const rightSidebar = document.querySelector('stencila-right-sidebar')
      const layoutEl = document.querySelector('stencila-layout')
      getPageCache().set(normalizeUrl(window.location.href), {
        title: document.title,
        mainHTML: mainElement.innerHTML,
        leftSidebarHTML: leftSidebar?.innerHTML,
        rightSidebarHTML: rightSidebar?.innerHTML,
        mainWidth: layoutEl?.getAttribute('main-width') ?? undefined,
        mainPadding: layoutEl?.getAttribute('main-padding') ?? undefined,
        mainTitle: layoutEl?.getAttribute('main-title') ?? undefined,
        timestamp: Date.now(),
      })
    }
  }

  // Save initial scroll position
  saveScrollPosition(window.location.href)

  // Initialize prefetch manager
  const cleanupPrefetch = initPrefetch(config)

  // Set up event listeners
  document.addEventListener('click', handleClick)
  window.addEventListener('popstate', handlePopstate)
  window.addEventListener(GLIDE_REQUEST, handleGlideRequest as EventListener)

  cleanup = () => {
    document.removeEventListener('click', handleClick)
    window.removeEventListener('popstate', handlePopstate)
    window.removeEventListener(GLIDE_REQUEST, handleGlideRequest as EventListener)
    cleanupPrefetch()
    history.scrollRestoration = 'auto'
    cleanup = null
  }

  return cleanup
}
