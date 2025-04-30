import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/commands/chat-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { Math } from './math'

@customElement('stencila-math-block')
@withTwind()
export class MathBlock extends Math {
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
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
        type="MathBlock"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${hasRoot}
      >
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="MathBlock"
            code=${this.code}
            node-id=${this.id}
            .code-authorship=${this.codeAuthorship}
            language=${this.mathLanguage ?? 'tex'}
            ?read-only=${!hasRoot}
          >
            <slot name="compilation-messages" slot="messages"></slot>
          </stencila-ui-node-code>
        </div>

        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderContent() {
    return html`
      <div class="px-4 py-6 text-base flex justify-center">
        <slot name="mathml"></slot>
        <slot name="images"></slot>
      </div>
    `
  }
}
