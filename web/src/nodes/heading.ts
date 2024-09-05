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
   * Get the top of the headings bounding client rectangle
   * 
   * Used for updating the list of visible headings.
   */
  getRectTop() {
    return this.getBoundingClientRect().top
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
