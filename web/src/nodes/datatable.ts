import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/card'

import { Entity } from './entity'
import './datatable-column'

/**
 * Web component representing a Stencila Schema `Datatable` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable.md
 */
@customElement('stencila-datatable')
export class Datatable extends Entity {
  /**
   * In static view just render the table
   */
  override renderStaticView() {
    return html`<slot></slot>`
  }

  /**
   * In dynamic view, in addition to the table, render a node card.
   */
  override renderDynamicView() {
    return html`<stencila-ui-node-card
      type="Datatable"
      view="dynamic"
      ?collapsible=${true}
      ><div slot="body"><slot></slot></div
    ></stencila-ui-node-card>`
  }

  /**
   * In source view, render the same as dynamic view, including the
   * table since that won't be a present in the source usually (given this
   * node type is normally only present in `CodeChunk.outputs`).
   */
  override renderSourceView() {
    return html`<stencila-ui-node-card
      type="Datatable"
      view="source"
      ?collapsible=${true}
      ><div slot="body"><slot></slot></div
    ></stencila-ui-node-card>`
  }
}
