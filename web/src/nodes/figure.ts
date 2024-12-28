import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Figure`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/figure.md
 */
@customElement('stencila-figure')
@withTwind()
export class Figure extends Entity {
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    return html`
      <stencila-ui-block-on-demand
        type="Figure"
        node-id=${this.id}
        depth=${this.depth}
      >
        <div slot="header-right">
          <stencila-ui-node-chat-commands
            type="Figure"
            node-id=${this.id}
            depth=${this.depth}
          >
          </stencila-ui-node-chat-commands>
        </div>

        <div slot="body">
          <stencila-ui-node-authors type="Figure">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>

        ${this.renderContent()}
      </stencila-ui-block-on-demand>
    `
  }

  private renderContent() {
    return html`
      <figure class="m-0">
        <slot name="content"></slot>
      </figure>
    `
  }
}
