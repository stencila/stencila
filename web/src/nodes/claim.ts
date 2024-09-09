import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Claim` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/claim.md
 */
@customElement('stencila-claim')
@withTwind()
export class Claim extends Entity {
  @property({ attribute: 'claim-type' })
  claimType: string

  /**
   * In dynamic view render `content`, and `authors` and summary stats in a node
   * card that is shown on hover.
   */
  override render() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand
        type="Claim"
        header-title=${this.claimType}
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Claim">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
