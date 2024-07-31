/**
 * Get the closest element matching a selector
 *
 * Use this in Web Component: it will search up and out of
 * the Shadow DOM, whereas `this.closest` will not.
 *
 * https://stackoverflow.com/a/67676665/4625911
 */
export function getClosest(
  node: Node | null,
  selector: string
): HTMLElement | null {
  if (!node) {
    return null
  }

  if (node instanceof ShadowRoot) {
    return getClosest(node.host, selector)
  }

  if (node instanceof HTMLElement) {
    if (node.matches(selector)) {
      return node
    } else {
      return getClosest(node.parentNode, selector)
    }
  }

  return getClosest(node.parentNode, selector)
}
