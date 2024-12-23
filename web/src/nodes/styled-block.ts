import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { Styled } from './styled'

/**
 * Web component representing a Stencila Schema `StyledBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/styled/styled-block.md
 */
@customElement('stencila-styled-block')
export class StyledBlock extends Styled {
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`<slot name="content"></slot>`
    }

    return html`<stencila-ui-block-on-demand
      type="StyledBlock"
      .canAnimate=${false}
      depth=${this.depth}
      ?isRootNode=${this.root}
    >
      <div slot="body">
        <stencila-ui-node-authors type="StyledBlock">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="StyledBlock"
          code=${this.code}
          .code-authorship=${this.codeAuthorship}
          language=${this.styleLanguage}
          read-only
        >
          <slot name="compilation-messages" slot="messages"></slot>
        </stencila-ui-node-code>
      </div>

      <slot name="content" slot="content"></slot>
    </stencila-ui-block-on-demand>`
  }
}
