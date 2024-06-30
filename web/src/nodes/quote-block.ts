import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

import '../ui/nodes/properties/provenance/provenance'

/**
 * Web component representing a Stencila Schema `QuoteBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote-block.md
 */
@customElement('stencila-quote-block')
@withTwind()
export class QuoteBlock extends Entity {
  /**
   * In dynamic view render `content` and summary stats in a node
   * card that is shown on hover.
   */
  override render() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand
        type="QuoteBlock"
        view="dynamic"
        node-id=${this.id}
      >
        <div slot="body">
          <stencila-ui-node-authors type="QuoteBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
