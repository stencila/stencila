import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { property, customElement, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { InstructionContext, instructionContext } from '../context'
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
  @consume({ context: instructionContext, subscribe: true })
  @state()
  context: InstructionContext

  /**
   * Number of authors who have edited this content.
   */
  @property()
  count: number

  /**
   * A stringified array, containing the numeric value for each author.
   */
  @property()
  authors: string

  /**
   * provenance string
   * one, or a combination of: 'Hw', 'He', 'Hv', 'Mw', 'Me', 'Mv'.
   */
  @property()
  provenance: string

  /**
   * 'Machine influence' rank,
   * number from 0 (human only) to 5 (machine only).
   */
  @property()
  mi: number

  override render() {
    const bgColour = getProvenanceHighlight(this.mi as ProvenanceHighlightLevel)

    if (this.context.cardOpen) {
      return html`
        <sl-tooltip
          style="--show-delay: 1000ms; background-color: white;"
          placement="bottom-start"
        >
          <!-- tooltip content -->
          <div slot="content">
            ${this.renderTooltipContent('Author count', this.count)}
            ${this.renderTooltipContent('Provenance', this.provenance)}
          </div>
          <!-- -->
          <span style="background-color: ${bgColour};">
            <slot></slot>
          </span>
        </sl-tooltip>
      `
    } else {
      return html`<slot></slot>`
    }
  }

  renderTooltipContent(property: string, value: string | number) {
    const styles = apply(['flex justify-between', 'text-sm'])
    return html`
      <div class=${styles}>
        <span class="font-bold mr-2">${property}: </span>
        <span>${value}</span>
      </div>
    `
  }
}
