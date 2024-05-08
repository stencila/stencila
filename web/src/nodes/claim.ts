import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'

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
  /**
   * In static view just render the `content`.
   */
  override renderStaticView() {
    return html`<slot name="content"></slot>`
  }

  /**
   * In dynamic view render `content`, and `authors` and summary stats in a node
   * card that is shown on hover.
   */
  override renderDynamicView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand
        type="Claim"
        title=${`Claim: ${this.claimType}`}
        view="dynamic"
      >
        <div slot="body">
          <stencila-ui-node-authors type="Claim">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In source view render `authors` and summary stats in a node card. Do not render
   * `content` since that is visible in the source code.
   */
  override renderSourceView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand type="Claim" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="Claim">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
