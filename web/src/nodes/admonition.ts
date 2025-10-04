import { AdmonitionType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import { booleanConverter } from '../utilities/booleanConverter'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

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
  @property({ attribute: 'is-folded', converter: booleanConverter })
  isFolded?: boolean

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    return this.renderCard()
  }

  override renderCard() {
    return html`
      <stencila-ui-block-on-demand
        type="Admonition"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Admonition">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>

        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderContent() {
    return html`<slot></slot>`
  }
}
