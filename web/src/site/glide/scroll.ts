/**
 * Scroll utilities for client-side navigation
 *
 * CSS scroll-padding-top is set in layout.css for native anchor navigation.
 * For programmatic scrolling, we center elements to avoid header issues.
 */

/**
 * Scroll to an element, centering it in the viewport
 *
 * Centers the element vertically to avoid issues with fixed headers
 * and provide good visibility of surrounding context.
 *
 * @param element - Element to scroll to
 * @param behavior - Scroll behavior ('auto' or 'smooth')
 */
export function scrollToElement(
  element: Element,
  behavior: ScrollBehavior = 'auto'
): void {
  element.scrollIntoView({ behavior, block: 'center' })
}

/**
 * Scroll to an element by ID, centering it in the viewport
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
