import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/properties/provenance/provenance'

/**
 * Figure
 *
 * Stencila Figure Entity
 */
@customElement('stencila-figure')
@withTwind()
export class Figure extends Entity {
  /**
   * In static view just render the figure
   */
  override renderStaticView() {
    return html`<slot></slot>`
  }

  /**
   * In dynamic view render `content`, and `authors` and summary stats in a node
   * card that is shown on hover.
   */
  override renderDynamicView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand
        type="Figure"
        view="dynamic"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Figure">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="Figure">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
        <figure slot="content" class="m-0">
          <slot name="content"></slot>
        </figure>
      </stencila-ui-block-on-demand>
    `
  }
}
