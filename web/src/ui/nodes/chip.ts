import { IntersectionController } from '@lit-labs/observers/intersection-controller'
import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

import { nodeUi } from './icons-and-colours'
import { NodeColours } from './mixins/toggle-chip'

const OFFSET_THRESHOLD = 10
const OFFSET = 10

@customElement('stencila-ui-node-chip')
@withTwind()
export class UINodeChip extends LitElement {
  @property({ type: Object })
  colours: NodeColours

  @property({ type: String })
  type: NodeType

  /**
   * Display type of node: 'block' or 'inline'
   */
  @property({ type: String, attribute: 'node-display' })
  nodeDisplay: 'inline' | 'block' = 'block'

  /**
   *
   */
  @property({ type: Boolean, attribute: 'card-open' })
  cardOpen: boolean

  /**
   * Function to fire on upon clicking the chip
   */
  @property({ type: Function })
  clickEvent: () => void

  /**
   *
   */
  static visibleChips: UINodeChip[] = []

  /**
   * Intersection observer to detect when a chip enters a view port
   */
  // @ts-expect-error observer is never read
  private observer = new IntersectionController(this, {
    callback: ([entry]) => {
      if (entry.isIntersecting) {
        this.enteredViewport(entry)
      } else {
        this.exitedViewport(entry)
      }
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
    const precedingChip =
      UINodeChip.visibleChips[UINodeChip.visibleChips.length - 1]

    const precedingChipTop = precedingChip?.getBoundingClientRect().top ?? null

    const thisTop = entry.boundingClientRect.top

    const diff = thisTop - precedingChipTop
    const absoluteDiff = Math.abs(diff)
    const isToClose = absoluteDiff <= OFFSET_THRESHOLD

    let outsideViewPort: boolean = false
    if (isToClose) {
      // apply offset if the absolute diffence in pixels is equal or
      // less than the threshold

      const realOffset = OFFSET - absoluteDiff

      if (diff > 0) {
        this.style.top = `${realOffset}px`
        const viewportHeight = entry.rootBounds.height
        if (this.getBoundingClientRect().top > viewportHeight) {
          outsideViewPort = true
        }
      } else if (diff < 0) {
        this.style.top = `-${realOffset}px`
        if (this.getBoundingClientRect().bottom < 0) {
          outsideViewPort = true
        }
      }
    }

    if (!outsideViewPort) {
      UINodeChip.visibleChips.push(this)
    }
    // console.log('Entered', UINodeChip.visibleChips)
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

    // console.log('Exited', UINodeChip.visibleChips)
  }

  override render() {
    const { colour, borderColour, textColour, icon } = nodeUi(this.type)
    const styles = apply([
      'h-8',
      'flex items-center',
      'transition duration-200',
      'leading-none',
      'px-2 py-1.5',
      `bg-[${colour}]`,
      `border rounded-md border-[${borderColour}]`,
      'cursor-pointer',
      `fill-black text-black`,
      `hover:bg-[${borderColour}] hover:border-[${colour}]`,
    ])

    return html`
      <div class=${`${styles}`} @click=${this.clickEvent}>
        <stencila-ui-icon
          name=${this.cardOpen ? 'chevronDown' : icon}
          class="text-base text-[${textColour}]"
        ></stencila-ui-icon>
      </div>
    `
  }
}
