import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila schema 'Table' node.
 */
@customElement('stencila-table')
@withTwind()
export class Table extends Entity {
  /**
   * render table and any additional content,
   * as well as `authors` inside a node card
   */
  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="Table"
        depth=${this.depth}
        ancestors=${this.ancestors}
        node-id=${this.id}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Table">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div class="content" slot="content">
          <slot name="caption"></slot>
          <div class="overflow-x-auto">
            <slot name="rows"></slot>
          </div>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
