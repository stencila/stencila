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
  ProvenanceOpacityLevel,
  getProvenanceOpacity,
} from '../../icons-and-colours'

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

  @state()
  protected toggleTooltip: boolean = true

  // Ensure that <stencila-authorship> element is inline
  static override styles = css`
    :host {
      display: inline;
      position: relative;
    }
  `

  override render() {
    if (
      this.previewContext.showAllAuthorshipHighlight ||
      this.entityContext.cardOpen
    ) {
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
          class="group relative text-[${computedColour}]/[${textOpacity}]"
        ><slot></slot
      ></span>`

    return htmlTemplate
  }
}
