import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance/provenance'

/**
 * Web component representing a Stencila schema 'Table' node.
 */
@customElement('stencila-table')
@withTwind()
export class Table extends Entity {
  /**
   * render table and any additional content
   */
  override renderStaticView() {
    return html`
      <slot name="content">
        <slot name="rows"></slot>
        <slot></slot>
      </slot>
    `
  }

  /**
   * render table and any additional content,
   * as well as `authors` inside a node card
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-block-on-demand type="Table" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="Table">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="Table">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
        <div class="content" slot="content">
          <div class="overflow-x-scroll">
            <slot name="rows"></slot>
          </div>
          <slot></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
