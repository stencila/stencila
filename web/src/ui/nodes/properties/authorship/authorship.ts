import { consume } from '@lit/context'
import { ProvenanceCategory } from '@stencila/types'
import { LitElement, html, css } from 'lit'
import { property, customElement, state } from 'lit/decorators'

import {
  documentPreviewContext,
  DocPreviewContext,
} from '../../../../contexts/preview-context'
import { withTwind } from '../../../../twind'
import { entityContext, EntityContext } from '../../context'
import {
  ProvenanceHighlightLevel,
  getProvenanceHighlight,
} from '../../icons-and-colours'

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

  @consume({ context: documentPreviewContext, subscribe: true })
  @state()
  previewContext: DocPreviewContext

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
  static override styles = css`
    :host {
      display: inline;
    }
  `

  override render() {
    const bgColour = getProvenanceHighlight(this.mi as ProvenanceHighlightLevel)

    if (
      this.previewContext.showAllAuthorshipHighlight ||
      this.entityContext.cardOpen
    ) {
      return html`
        <sl-tooltip
          style="--show-delay: 1000ms; display: inline-block;"
          placement="bottom-start"
          content=${getTooltipContent(this.count, this.provenance)}
        >
          <span style="background-color: ${bgColour}; display: inline-block;"
            ><slot></slot
          ></span>
        </sl-tooltip>
      `
    } else {
      return html`<slot></slot>`
    }
  }
}
