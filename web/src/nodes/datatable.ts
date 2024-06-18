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
   * In dynamic view, render a node card with the table in the content slot.
   */
  override render() {
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
}
