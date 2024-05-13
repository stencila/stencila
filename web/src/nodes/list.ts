import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../ui/nodes/card'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `List` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md
 */
@customElement('stencila-list')
export class List extends Entity {
  /**
   * In static view, just render the `items`.
   */
  override renderStaticView() {
    return html`<slot name="items"></slot>`
  }

  /**
   * In dynamic view, render the `items`, `authors` and summary stats in
   * a node card that is shown on demand.
   */
  override renderDynamicView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand type="List" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="List">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="List">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
        <slot name="items" slot="content"></slot>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In source view, render `authors` and summary stats in a node card. Do not render
   * `items` since those are visible in the source code.
   */
  override renderSourceView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-node-card type="List" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="List">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="List">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
      </stencila-ui-node-card>
    `
  }
}
