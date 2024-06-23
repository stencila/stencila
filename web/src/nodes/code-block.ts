import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance/provenance'

import { CodeStatic } from './code-static'

@customElement('stencila-code-block')
export class CodeBlock extends CodeStatic {
  /**
   * In dynamic view, also render the authors
   */
  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="CodeBlock"
        view="dynamic"
        programming-language=${this.programmingLanguage}
      >
        <div slot="body">
          <stencila-ui-node-authors type="CodeBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content">
          <stencila-ui-node-code
            type="CodeBlock"
            code=${this.code}
            code-authorship=${this.codeAuthorship}
            language=${this.programmingLanguage}
            read-only
          >
          </stencila-ui-node-code>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
