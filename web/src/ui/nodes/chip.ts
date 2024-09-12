import { IntersectionController } from '@lit-labs/observers/intersection-controller'
import { LitElement } from 'lit'
import { customElement } from 'lit/decorators'

@customElement('stencila-ui-node-chip')
export class UINodeChip extends LitElement {
  static visibleChips: UINodeChip[] = []

  /**
   * Intersection observer to detect when a chip enters a view port
   */
  // @ts-expect-error observer is never read
  private observer = new IntersectionController(this, {
    callback: ([entry]) => {
      entry.isIntersecting
        ? this.enteredViewport(entry)
        : this.exitedViewport(entry)
    },
  })

  /**
   * Handler for when this chip enters the viewport
   *
   * Note that this gets called on initial render for
   * all chips in the viewport and again when the chip
   * enters the viewport (due to scrolling up or down
   * or window resizing).
   */
  enteredViewport(entry: IntersectionObserverEntry) {
    // TODO: Move the node up or down if it overlaps with
    // any nodes in `visibleChips`. Note that this may push
    // the chip outside of the viewport so do not
    // add to `visibleChips` is that is the case.

    UINodeChip.visibleChips.push(this)

    console.log('Entered', UINodeChip.visibleChips)
  }

  /**
   * Handler for when this chip exits the viewport
   *
   * Note that this gets called on initial render for
   * all chips outside the viewport and again when the chip
   * exists the viewport (due to scrolling up or down
   * or window resizing).
   */
  exitedViewport(entry: IntersectionObserverEntry) {
    // If any of the node chip is still showing then do nothing
    if (entry.intersectionRatio > 0) {
      return
    }

    // Otherwise. Remove this chip from `visibleChips`
    const index = UINodeChip.visibleChips.indexOf(this)
    if (index > -1) {
      UINodeChip.visibleChips.splice(index, 1)
    }

    console.log('Exited', UINodeChip.visibleChips)
  }
}
