/**
 * Select the closest element matching a selector
 *
 * This is similar to the `closest` method of HTML elements but traverses
 * up out of the shadow root is necessary.
 *
 * Based on https://stackoverflow.com/questions/54520554/custom-element-getrootnode-closest-function-crossing-multiple-parent-shadowd
 */
export function closestGlobally(elem: HTMLElement, selector: string): HTMLElement | null {
  function closest(elem: HTMLElement | Document | Window): HTMLElement | null {
    if (!elem || elem === document || elem === window) return null
    const found = (elem as HTMLElement).closest(selector)
    // @ts-expect-error because `Node` has no host property
    return found ? found : closest(elem.getRootNode().host)
  }
  return closest(elem)
}
