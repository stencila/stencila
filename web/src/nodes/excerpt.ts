import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Excerpt`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/excerpt.md
 */
@customElement('stencila-excerpt')
@withTwind()
export class Excerpt extends Entity {
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithChatAction()
    }

    return this.renderCard()
  }

  private renderContent() {
    return html`
      <div>
        <slot name="content"></slot>
      </div>
    `
  }

  override renderCard() {
    const hasDocRoot = this.hasDocumentRootNode()

    return html`
      <stencila-ui-block-on-demand
        type="Excerpt"
        node-id=${this.id}
        depth=${this.depth}
        ?no-root=${!hasDocRoot}
      >
        <div slot="body">
          <slot name="source"></slot>
        </div>
        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }
}
