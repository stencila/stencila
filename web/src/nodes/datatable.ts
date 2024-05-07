import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/node-card/on-demand/block'
import './datatable-column'

/**
 * Web component representing a Stencila Schema `Datatable` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable.md
 */
@customElement('stencila-datatable')
@withTwind()
export class Datatable extends Entity {
  /**
   * In static view just render the table
   */
  override renderStaticView() {
    return html` <div class="overflow-x-scroll data-table" slot="content">
      <slot></slot>
    </div>`
  }

  /**
   * In dynamic view, render a node card with the table in the content slot.
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-block-on-demand type="Datatable" view="dynamic">
        <div class="content" slot="content">
          <div class="overflow-x-scroll data-table">
            <slot></slot>
          </div>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In source view, render the same as dynamic view, including the
   * table since that won't be a present in the source usually (given this
   * node type is normally only present in `CodeChunk.outputs`).
   */
  override renderSourceView() {
    return html`
      <stencila-ui-block-on-demand type="Datatable" view="source">
        <div slot="body">
          <div class="overflow-auto">
            <slot></slot>
          </div>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
