import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `QuoteBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/quote-block.md
 */
@customElement('stencila-quote-block')
@withTwind()
export class QuoteBlock extends Entity {
  override render() {
    if (this.ancestors.includes('StyledBlock') || this.isUserChatNode()) {
      return html`<slot name="content"></slot>`
    }

    return html`
      <stencila-ui-block-on-demand
        type="QuoteBlock"
        depth=${this.depth}
        ancestors=${this.ancestors}
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
