import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { withTwind } from '../twind'

import { Styled } from './styled'

/**
 * Web component representing a Stencila Schema `StyledBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/style/page.md
 */
@customElement('stencila-page')
@withTwind()
export class Page extends Styled {
  override render() {
    if (this.isWithinUserChatMessage()) {
      return html`<slot></slot>`
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithChatAction()
    }

    return this.renderCard()
  }

  override renderCard() {
    const hasRoot = this.hasRoot()

    return html`
      <stencila-ui-block-on-demand
        type="Page"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${hasRoot}
        no-content-padding
      >
        <div slot="body">
          <stencila-ui-node-authors type="Page">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="Page"
            node-id=${this.id}
            code=${this.code}
            .code-authorship=${this.codeAuthorship}
            language=${this.styleLanguage}
            ?read-only=${!hasRoot}
          >
          </stencila-ui-node-code>
        </div>

        <div slot="content">
          <slot></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
