import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `List` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md
 */
@customElement('stencila-list')
@withTwind()
export class List extends Entity {
  override render() {
    // Do not render a node card for document headings slot or StyledBlock
    if (
      this.closestGlobally('nav[slot=headings]') ||
      this.isWithin('StyledBlock') ||
      this.isWithinUserChatMessage()
    ) {
      return html`<slot name="items"></slot>`
    }

    if (this.isWithinModelChatMessage()) {
      return html`
        <div class="group relative">
          ${this.renderInsertChip()}
          <slot name="items"></slot>
        </div>
      `
    }

    return html`
      <stencila-ui-block-on-demand
        type="List"
        node-id=${this.id}
        depth=${this.depth}
      >
        <div slot="body">
          <stencila-ui-node-authors type="List">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>

        <slot name="items" slot="content"></slot>
      </stencila-ui-block-on-demand>
    `
  }
}
