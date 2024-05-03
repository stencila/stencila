import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

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
      <stencila-ui-block-on-demand type="Figure" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="Figure">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content">
          <figure>
            <slot name="content"></slot>
          </figure>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
