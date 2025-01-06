import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Table`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/table.md
 */
@customElement('stencila-table')
@withTwind()
export class Table extends Entity {
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    return html`
      <stencila-ui-block-on-demand
        type="Table"
        node-id=${this.id}
        depth=${this.depth}
      >
        <div slot="header-right">
          <stencila-ui-node-chat-commands
            type="Table"
            node-id=${this.id}
            depth=${this.depth}
          >
          </stencila-ui-node-chat-commands>
        </div>

        <div slot="body">
          <stencila-ui-node-authors type="Table">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>

        <div class="content" slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderContent() {
    return html`
      <slot name="caption"></slot>
      <div class="overflow-x-auto">
        <slot name="rows"></slot>
      </div>
    `
  }
}
