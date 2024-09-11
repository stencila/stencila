import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

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

  override render() {
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
