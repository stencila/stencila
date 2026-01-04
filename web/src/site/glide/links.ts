/**
 * Shared link utilities for navigation and prefetch
 */

/**
 * Normalize a URL by removing the hash fragment
 *
 * This ensures that different hash targets on the same page share a cache entry.
 *
 * @param url - URL to normalize (can be relative or absolute)
 * @returns Normalized URL with origin, pathname, and search (no hash)
 */
export function normalizeUrl(url: string): string {
  const parsed = new URL(url, window.location.origin)
  return parsed.origin + parsed.pathname + parsed.search
}

/**
 * Check if a link is eligible for client-side navigation
 *
 * A link is eligible if:
 * - It has an href
 * - It's same-origin
 * - It doesn't have data-stencila-glide="off"
 * - It's not a download link
 * - It doesn't have a target other than _self
 * - It uses http: or https: protocol
 *
 * @param link - The anchor element to check
 * @returns true if the link is eligible for client-side navigation
 */
export function isEligibleLink(link: HTMLAnchorElement): boolean {
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
 * Check if a link is eligible for prefetching
 *
 * Extends isEligibleLink with an additional check for data-stencila-prefetch="off"
 *
 * @param link - The anchor element to check
 * @returns true if the link is eligible for prefetching
 */
export function isEligibleForPrefetch(link: HTMLAnchorElement): boolean {
  // Skip if link has data-stencila-prefetch="off"
  if (link.dataset.stencilaPrefetch === 'off') {
    return false
  }

  return isEligibleLink(link)
}
