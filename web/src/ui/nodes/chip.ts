import { IntersectionController } from '@lit-labs/observers/intersection-controller'
import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

import { nodeUi } from './icons-and-colours'

const OFFSET = 15

@customElement('stencila-ui-node-chip')
@withTwind()
export class UINodeChip extends LitElement {
  @property({ type: String })
  type: NodeType

  /**
   * Display type of node: 'block' or 'inline'
   */
  @property({ type: String, attribute: 'node-display' })
  nodeDisplay: 'inline' | 'block' = 'block'

  /**
   * 'true' if the corresponding node card is open
   */
  @property({ type: Boolean, attribute: 'card-open' })
  cardOpen: boolean

  /**
   * Function to fire on upon clicking the chip
   */
  @property({ type: Function })
  clickEvent: () => void

  /**
   * Global list of the chip elements within the viewport
   */
  static visibleChips: UINodeChip[] = []

  /**
   * Intersection observer to detect when a chip enters a view port
   */
  // @ts-expect-error observer is never read
  private observer = new IntersectionController(this, {
    callback: ([entry]) => {
      if (entry.isIntersecting) {
        this.enteredViewport()
      } else {
        this.exitedViewport(entry)
      }
    },
  })

  /**
   * Calculate the gap in pixels between `this` and another node chip.
   */
  private getGapBetweenChips(chip: UINodeChip) {
    const thisTop = this.getBoundingClientRect().top
    const otherTop = chip.getBoundingClientRect().top
    return Math.abs(thisTop - otherTop)
  }

  /**
   * Handler for when this chip enters the viewport
   *
   * Note that this gets called on initial render for
   * all chips in the viewport and again when the chip
   * enters the viewport (due to scrolling up or down
   * or window resizing).
   */
  enteredViewport() {
    // if visible chips has elements already
    if (UINodeChip.visibleChips.length > 0) {
      // copy chip array <- do not mutate the global value unless we have too
      const chipsArray = UINodeChip.visibleChips
      // add `this`
      chipsArray.push(this)
      // sort array by order on page
      chipsArray.sort(
        (a, b) => a.getBoundingClientRect().top - b.getBoundingClientRect().top
      )
      // get the index of the chip
      const index = chipsArray.indexOf(this)

      // assign prev and next chips (values will be null if they don't exist)
      const prevChip = index > 0 ? chipsArray[index - 1] : null
      const nextChip =
        index < chipsArray.length - 1 ? chipsArray[index + 1] : null

      let offsetApplied = false
      // if scrolling up `prevChip` should be undefined, as `this` will be first element in the sorted array
      if (prevChip) {
        const gap = this.getGapBetweenChips(prevChip)
        if (gap < OFFSET) {
          const realOffset = OFFSET - gap
          this.style.top = `${realOffset}px` // offset 'down'
          offsetApplied = true
        }
      }
      // if no offset has been applied (or no previous chip exists), check the `nextChip`
      if (nextChip && !offsetApplied) {
        const gap = this.getGapBetweenChips(nextChip)
        if (gap < OFFSET) {
          const realOffset = OFFSET - gap
          this.style.top = `-${realOffset}px` // offset 'up'
          offsetApplied = true
        }
      }

      if (offsetApplied) {
        const rect = this.getBoundingClientRect()
        const viewportHeight =
          window.innerHeight || document.documentElement.clientHeight

        if (rect.top >= viewportHeight && rect.bottom <= 0) {
          // if chip is now outside the bounds,
          // return before appending this chip to the array
          return
        }
      }
    }
    UINodeChip.visibleChips.push(this)
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
