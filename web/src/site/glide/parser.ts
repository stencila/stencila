/**
 * HTML parsing and content extraction for client-side navigation
 *
 * Uses DOMParser to extract page content from fetched HTML without
 * inserting it into the live document.
 */

import type { CacheEntry } from './types'

/**
 * Parse fetched HTML and extract content for caching/display
 *
 * @param html - Raw HTML string from fetch response
 * @param contentSelector - CSS selector for main content element
 * @returns Parsed cache entry, or null if content element not found
 */
export function parseHTML(html: string, contentSelector: string): CacheEntry | null {
  const parser = new DOMParser()
  const doc = parser.parseFromString(html, 'text/html')

  // Extract title
  const title = doc.title || ''

  // Extract main content
  const mainElement = doc.querySelector(contentSelector)
  if (!mainElement) {
    return null
  }
  const mainHTML = mainElement.innerHTML

  // Extract sidebar content (undefined if sidebar not present)
  const leftSidebar = doc.querySelector('stencila-left-sidebar')
  const rightSidebar = doc.querySelector('stencila-right-sidebar')
  const leftSidebarHTML = leftSidebar?.innerHTML
  const rightSidebarHTML = rightSidebar?.innerHTML

  // Extract meta description (optional)
  const metaDesc = doc.querySelector('meta[name="description"]')
  const metaDescription = metaDesc?.getAttribute('content') ?? undefined

  // Extract canonical URL (optional)
  const canonicalLink = doc.querySelector('link[rel="canonical"]')
  const canonical = canonicalLink?.getAttribute('href') ?? undefined

  return {
    title,
    mainHTML,
    leftSidebarHTML,
    rightSidebarHTML,
    metaDescription,
    canonical,
    timestamp: Date.now(),
  }
}

/**
 * Extract just the main content HTML from a document
 *
 * Convenience wrapper around parseHTML when only content is needed.
 *
 * @param html - Raw HTML string
 * @param contentSelector - CSS selector for main content
 * @returns Inner HTML of content element, or null if not found
 */
export function extractContent(html: string, contentSelector: string): string | null {
  return parseHTML(html, contentSelector)?.mainHTML ?? null
}
