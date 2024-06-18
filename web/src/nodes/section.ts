import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

import '../ui/nodes/properties/provenance/provenance'

/**
 * Web component representing a Stencila Schema `Section` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md
 */
@customElement('stencila-section')
@withTwind()
export class Section extends Entity {
  /**
   * render `content` and summary stats in a node
   * card that is shown on hover.
   */
  override render() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand type="Section" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="Section">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="Section">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
