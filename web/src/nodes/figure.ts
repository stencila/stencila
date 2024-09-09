import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/properties/provenance'

/**
 * Figure
 *
 * Stencila Figure Entity
 */
@customElement('stencila-figure')
@withTwind()
export class Figure extends Entity {
  /**
   * In dynamic view render `content`, and `authors` and summary stats in a node
   * card that is shown on hover.
   */
  override render() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand
        type="Figure"
        depth=${this.depth}
        ancestors=${this.ancestors}
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
