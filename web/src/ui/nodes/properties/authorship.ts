import { consume } from '@lit/context'
import { ProvenanceCategory } from '@stencila/types'
import { apply, css } from '@twind/core'
import { LitElement, html } from 'lit'
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
  provenance: ProvenanceCategory

  /**
   * 'Machine influence' rank,
   * number from 0 (human only) to 5 (machine only).
   */
  @property()
  mi: number

  override render() {
    const bgColour = getProvenanceHighlight(this.mi as ProvenanceHighlightLevel)

    const toolTipStyles = css`
      &::part(body) {
        color: #000000;
        background-color: ${getProvenanceHighlight(2)};
      }
      &::part(base__arrow) {
        background-color: ${getProvenanceHighlight(2)};
      }
    `

    if (this.context.cardOpen) {
      return html`
        <sl-tooltip
          style="--show-delay: 1000ms; background-color: white;"
          placement="bottom-start"
          class=${toolTipStyles}
        >
          <!-- tooltip content -->
          <div slot="content">
            ${this.renderTooltipContent(
              `${this.count} Author${this.count > 1 ? 's' : ''}`
            )}
            ${this.renderTooltipContent(this.provenance)}
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

  renderTooltipContent(value: string | number) {
    const styles = apply(['flex justify-between', 'text-sm'])
    return html` <div class=${styles}>${value}</div> `
  }
}
