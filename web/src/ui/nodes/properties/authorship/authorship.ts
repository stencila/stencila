import { consume } from '@lit/context'
import { ProvenanceCategory } from '@stencila/types'
import { LitElement, html, css, PropertyValueMap } from 'lit'
import { property, customElement, state } from 'lit/decorators'
import { Ref, createRef, ref } from 'lit/directives/ref'

import { withTwind } from '../../../../twind'
import { documentContext, DocumentContext } from '../../../document/context'
import { entityContext, EntityContext } from '../../context'
import {
  ProvenanceOpacityLevel,
  getProvenanceOpacity,
} from '../../icons-and-colours'

import { AuthorshipTooltip } from './tooltip'
import { getTooltipContent } from './utils'

/**
 * Renders the author provenance highlighting of document text.
 */
@customElement('stencila-authorship')
@withTwind()
export class StencilaAuthorship extends LitElement {
  @consume({ context: entityContext, subscribe: true })
  @state()
  entityContext: EntityContext

  @consume({ context: documentContext, subscribe: true })
  @state()
  documentContext: DocumentContext

  /**
   * Number of authors who have ever edited this content.
   *
   * Note that this may be
   */
  @property({ type: Number })
  count: number

  /**
   * A stringified array, containing the 0-based index of the
   * author in the closes parent element with an `authors` slot.
   */
  @property({ type: Array })
  authors: number[]

  /**
   * Provenance description
   *
   * One, or a combination of, 'Hw', 'He', 'Hv', 'Mw', 'Me', 'Mv',
   * and including numeric prefixes for multiple verifications e.g Hv
   */
  @property()
  provenance: ProvenanceCategory

  /**
   * 'Machine influence' rank.
   *
   * A number from 0 (human only) to 5 (machine only).
   */
  @property({ type: Number })
  mi: number

  @state()
  protected toggleTooltip: boolean = true

  // Ensure that <stencila-authorship> element is inline
  static override styles = css`
    :host {
      display: inline;
      position: relative;
    }
  `

  /**
   * element refs for tooltip functionality
   */
  private authorshipRef: Ref<HTMLElement> = createRef()
  private tooltipRef: Ref<AuthorshipTooltip> = createRef()

  /**
   * Calculate the x and y positions for the tooltip
   * @returns `{ x: number; y: number }`
   */
  private tooltipPosition(): { x: number; y: number } {
    const { x, y, width } = this.authorshipRef.value.getBoundingClientRect()
    return { x: x + width / 2, y }
  }

  override update(
    changedProperties: PropertyValueMap<this> | Map<PropertyKey, unknown>
  ) {
    super.update(changedProperties)

    if (this.authorshipRef.value) {
      this.authorshipRef.value.addEventListener('mouseover', () => {
        const { x, y } = this.tooltipPosition()
        this.tooltipRef.value.xPos = x
        this.tooltipRef.value.yPos = y
        this.tooltipRef.value.open = true
      })

      this.authorshipRef.value.addEventListener('mouseout', () => {
        this.tooltipRef.value.open = false
      })
    }
  }

  override render() {
    const showHighlights =
      this.documentContext?.showAllAuthorshipHighlight ||
      this.entityContext?.cardOpen

    if (showHighlights) {
      return this.renderHighlights()
    } else {
      return html`<slot></slot>`
    }
  }

  renderHighlights() {
    const textOpacity = getProvenanceOpacity(this.mi as ProvenanceOpacityLevel)

    // find the current text color in rgb (remove all whitespace)
    const computedColour = window
      .getComputedStyle(this)
      .getPropertyValue('color')
      .replace(/\s/g, '')

    /*
      Do not change the formatting of this template,
      line breaks between tags will introduce 
      whitespace into the text in the document preview.
    */
    // prettier-ignore
    const htmlTemplate = html`<span
          ${ref(this.authorshipRef)}
          class="group relative text-[${computedColour}]/[${textOpacity}]"
        ><slot></slot
      ></span><authorship-tooltip ${ref(this.tooltipRef)} content=${getTooltipContent(this.count, this.provenance)}></authorship-tooltip>`

    return htmlTemplate
  }
}
