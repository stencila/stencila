import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/card'
import '../ui/nodes/node-card/in-flow/block'
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
  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with authors and code read-only.
   */
  override render() {
    this.adoptCss()

    return html`<stencila-ui-block-on-demand
      type="StyledBlock"
      .canAnimate=${false}
      depth=${this.depth}
      ancestors=${this.ancestors}
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
        </stencila-ui-node-code>
      </div>

      <div slot="content" class="styled">
        <div class="${this.classes}">
          <slot name="content"></slot>
        </div>
      </div>
    </stencila-ui-block-on-demand>`
  }
}
