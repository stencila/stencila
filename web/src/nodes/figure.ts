import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { createCaptionLabel } from '../ui/nodes/properties/captions'

import { Entity } from './entity'

import '../ui/nodes/properties/captions/caption-label'
import '../ui/nodes/properties/provenance/provenance'

/**
 * Figure
 *
 * Stencila Figure Entity
 */
@customElement('stencila-figure')
@withTwind()
export class Figure extends Entity {
  @property()
  label: string

  override connectedCallback(): void {
    super.connectedCallback()

    createCaptionLabel(this, 'Figure')
  }

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
          <stencila-ui-node-provenance type="Figure">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
        <div slot="content" class="m-0">
          <slot name="content"></slot>
          <figcaption>
            <slot name="caption" slot="caption"></slot>
          </figcaption>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
