import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/commands/clone-commands'
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

    return html`
      <stencila-ui-block-on-demand
        type="MathBlock"
        node-id=${this.id}
        depth=${this.depth}
      >
        <span slot="header-right">
          <stencila-ui-node-clone-commands
            type="MathBlock"
            node-id=${this.id}
            ?enabled=${this.isWithin('Chat')}
          >
          </stencila-ui-node-clone-commands>
        </span>

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
            .code-authorship=${this.codeAuthorship}
            language=${this.mathLanguage ?? 'tex'}
            read-only
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
      <div class="px-4 py-3">
        <slot name="mathml"></slot>
      </div>
    `
  }
}
