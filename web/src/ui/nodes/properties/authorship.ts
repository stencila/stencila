import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { property, customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { ProvenanceHighlightLvl, getProvHighlight } from '../icons-and-colours'

@customElement('stencila-authorship')
@withTwind()
export class StencilaAuthorship extends LitElement {
  @property()
  count: number

  @property()
  authors: string

  @property()
  provenance: string

  @property()
  mi: number

  override render() {
    const bgColour = getProvHighlight(this.mi as ProvenanceHighlightLvl)

    return html`
      <sl-tooltip style="--show-delay: 1000ms;">
        <div slot="content">
          ${this.renderTooltipContent('Author count', this.count)}
          ${this.renderTooltipContent('Provenance', this.provenance)}
        </div>
        <span style="background-color: ${bgColour}">
          <slot></slot>
        </span>
      </sl-tooltip>
    `
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
