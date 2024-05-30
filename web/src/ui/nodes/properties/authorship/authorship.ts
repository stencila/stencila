import { consume } from '@lit/context'
import { ProvenanceCategory } from '@stencila/types'
import { apply } from '@twind/core'
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
    const textOpacity = getProvenanceHighlight(
      this.mi as ProvenanceHighlightLevel
    )
    const tooltipStyle = apply([
      'absolute bottom-[calc(100%+0.5rem)] left-1/2 z-10',
      'group-hover:opacity-100',
      'w-32',
      'opacity-0',
      'rounded',
      'p-2',
      'transition-all delay-200 duration-300',
      'text-white text-sm',
      'bg-black',
      'transform -translate-x-1/2',
      'pointer-events-none',
      'after:content[""]',
      'after:absolute after:-bottom-1 after:left-1/2',
      'after:w-2 after:h-2',
      'after:bg-black',
      'after:transform after:-translate-x-1/2 after:rotate-45',
    ])

    /*
      Do not change the formatting of this template,
      line breaks between tags will introduce 
      whitespace into the text in the document preview.
    */
    // prettier-ignore
    const htmlTemplate = html`<span
          class="group text-black"
          style="position: relative; --tw-text-opacity: ${textOpacity};"
        ><div class=${tooltipStyle}>${getTooltipContent(this.count, this.provenance)}</div
        ><slot></slot
      ></span>`

    return htmlTemplate
  }
}
