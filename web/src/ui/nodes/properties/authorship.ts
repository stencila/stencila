import { consume } from '@lit/context'
import { ProvenanceCategory } from '@stencila/types'
import { LitElement, html, css } from 'lit'
import { property, customElement, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { entityContext, EntityContext } from '../context'
import {
  ProvenanceHighlightLevel,
  getProvenanceHighlight,
} from '../icons-and-colours'

/**
 * Renders the author provenance highlighting of document text.
 */
@customElement('stencila-authorship')
@withTwind()
export class StencilaAuthorship extends LitElement {
  @consume({ context: entityContext, subscribe: true })
  @state()
  context: EntityContext

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

  // Ensure that <stencila-authorship> element is inline
  // This does not fix issues with extra unwanted space around the element.
  // Some other CSS rules might.
  static override styles = css`
    :host {
      display: inline;
    }
  `

  override render() {
    const bgColour = getProvenanceHighlight(this.mi as ProvenanceHighlightLevel)

    // TODO: Enable tooltips in a way that does not introduce additional spacing
    // around the <stencila-authorship> element.
    /*
    const tooltipStyles = twCss`
      &::part(body) {
        color: #000000;
        background-color: ${getProvenanceHighlight(2)};
      }
      &::part(base__arrow) {
        background-color: ${getProvenanceHighlight(2)};
      }
    `

    let tooltipContent = `${this.count} author${this.count > 1 ? 's' : ''}`
    if (this.provenance.startsWith('Mw')) {
      tooltipContent += ', machine written'
    } else if (this.provenance.startsWith('Hw')) {
      tooltipContent += ', human written'
    }
    if (this.provenance.includes('Me')) {
      tooltipContent += ', machine edited'
    } else if (this.provenance.includes('He')) {
      tooltipContent += ', human edited'
    }
    if (this.provenance.includes('Mv')) {
      tooltipContent += ', machine verified'
    } else if (this.provenance.includes('Hv')) {
      tooltipContent += ', human verified'
    }
    */

    if (this.context.cardOpen) {
      return html`<span style="background-color: ${bgColour};"
        ><slot></slot
      ></span>`
      /*
      return html`<sl-tooltip
        style="--show-delay: 1000ms; background-color: white; display:inline;"
        placement="bottom-start"
        class=${tooltipStyles}
        content=${tooltipContent}
        ><span style="background-color: ${bgColour};"><slot></slot></span
      ></sl-tooltip>`
      */
    } else {
      return html`<slot></slot>`
    }
  }
}
