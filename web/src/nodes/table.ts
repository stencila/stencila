import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'

/**
 * Web component representing a Stencila schema 'Table' node.
 */
@customElement('stencila-table')
@withTwind()
export class Table extends Entity {
  override renderStaticView() {
    return html`<slot name="content"></slot>`
  }

  override renderDynamicView() {
    return html`
      <stencila-ui-block-on-demand type="Table" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="Table">
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
