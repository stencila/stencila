import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/authorship'
import '../ui/nodes/properties/provenance/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Heading` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md
 */
@customElement('stencila-heading')
export class Heading extends Entity {
  @property({ type: Number })
  level: number

  /**
   * Determine if the heading is currently visible
   *
   * Only considers vertical position in the viewport (it is
   * possible for this method to return `true` but that the heading is
   * horizontally outside of the viewport).
   */
  isVisible() {
    const rect = this.getBoundingClientRect()
    return (
      rect.top >= 0 &&
      rect.bottom <=
        (window.innerHeight || document.documentElement.clientHeight)
    )
  }

  /**
   * In dynamic view, render the `content`, `authors` and summary stats in
   * a node card that is shown on demand.
   */
  override render() {
    return html`
      <stencila-ui-block-on-demand type="Heading" node-id=${this.id}>
        <div slot="body">
          <stencila-ui-node-authors type="Heading">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <slot name="content" slot="content"></slot>
      </stencila-ui-block-on-demand>
    `
  }
}
