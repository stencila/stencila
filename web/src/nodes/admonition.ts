import { AdmonitionType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Admonition` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md
 */
@customElement('stencila-admonition')
@withTwind()
export class Admonition extends Entity {
  /**
   * The type of admonition.
   */
  @property({ attribute: 'admonition-type' })
  admonitionType: AdmonitionType

  /**
   * Whether the admonition is folded.
   */
  @property({ attribute: 'is-folded' })
  isFolded?: string

  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="Admonition"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Admonition">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <!-- TODO: Styling, including colours based on this.admonitionType -->
        <div slot="content" class="rounded border border-gray-500">
          <div class="rounded-t bg-gray-300 p-2">
            <!-- TODO: Icon based on the this.admonitionType -->
            <!-- TODO: If the title slot is empty then use the admonition type -->
            <slot name="title"></slot>
            <!-- TODO: Chevron if this.isFolded is defined, downward if false, right if true -->
          </div>
          <div>
            <!-- TODO: Show/hide content based on this.isFolded -->
            <slot name="content"></slot>
          </div>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
