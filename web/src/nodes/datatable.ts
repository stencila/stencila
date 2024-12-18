import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'

import { Entity } from './entity'

import './datatable-column'

/**
 * Web component representing a Stencila Schema `Datatable` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/datatable.md
 */
@customElement('stencila-datatable')
@withTwind()
export class Datatable extends Entity {
  override render() {
    if (
      this.ancestors.includes('StyledBlock') ||
      this.isUserChatMessageNode()
    ) {
      return this.renderContent()
    }

    return html`
      <stencila-ui-block-on-demand
        type="Datatable"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <div class="content" slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderContent() {
    return html`
      <div class="overflow-x-scroll data-table">
        <slot></slot>
      </div>
    `
  }
}
