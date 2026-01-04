/**
 * Scroll utilities for client-side navigation
 */

/** Fixed header offset in pixels */
export const HEADER_OFFSET = 80

/**
 * Scroll to an element with header offset
 *
 * @param element - Element to scroll to
 * @param behavior - Scroll behavior ('auto' or 'smooth')
 */
export function scrollToElement(
  element: Element,
  behavior: ScrollBehavior = 'auto'
): void {
  const elementPosition = element.getBoundingClientRect().top
  const offsetPosition = elementPosition + window.scrollY - HEADER_OFFSET
  window.scrollTo({ top: offsetPosition, behavior })
}

/**
 * Scroll to an element by ID with header offset
 *
 * @param id - Element ID (without #)
 * @param behavior - Scroll behavior
 * @returns true if element was found and scrolled to
 */
export function scrollToId(id: string, behavior: ScrollBehavior = 'auto'): boolean {
  const element = document.getElementById(id)
  if (element) {
    scrollToElement(element, behavior)
    return true
  }
  return false
}
