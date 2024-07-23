import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/node-card/on-demand/in-line'
import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance/provenance'

import { Styled } from './styled'

/**
 * Web component representing a Stencila Schema `StyledInline` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/styled/styled-inline.md
 */
@customElement('stencila-styled-inline')
export class StyledInline extends Styled {
  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with authors and code read-only.
   */
  override render() {
    this.adoptCss()

    return html` <stencila-ui-inline-on-demand
      type="StyledInline"
    >
      <div slot="body">
        <stencila-ui-node-authors type="StyledInline">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="StyledInline"
          code=${this.code}
          .code-authorship=${this.codeAuthorship}
          language=${this.styleLanguage}
          read-only
        >
        </stencila-ui-node-code>
      </div>

      <span slot="content" class="styled">
        <span class="${this.classes}">
          <slot name="content"></slot>
        </span>
      </span>
    </stencila-ui-inline-on-demand>`
  }
}
