import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

/**
 * Figure
 *
 * Stencila Figure Entity
 */
@customElement('stencila-figure')
@withTwind()
export class Figure extends Entity {
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`
        <figure class="m-0">
          <slot name="content"></slot>
        </figure>
      `
    }

    return html`
      <stencila-ui-block-on-demand
        type="Figure"
        depth=${this.depth}
        ?isRootNode=${this.root}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Figure">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <figure slot="content" class="m-0">
          <slot name="content"></slot>
        </figure>
      </stencila-ui-block-on-demand>
    `
  }
}
