import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../ui/nodes/card'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `List` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md
 */
@customElement('stencila-list')
export class List extends Entity {
  /**
   * render the `items`, `authors` and summary stats in
   * a node card that is shown on demand.
   */
  override render() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand
        type="List"
        view="dynamic"
        node-id=${this.id}
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
