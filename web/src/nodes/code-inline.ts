import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../ui/nodes/node-card/on-demand/in-line'
import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/outputs'

import { CodeStatic } from './code-static'

@customElement('stencila-code-inline')
export class CodeInline extends CodeStatic {
  /**
   * In dynamic view, also render the authors
   */
  override render() {
    return html`
      <stencila-ui-inline-on-demand
        type="CodeInline"
        view="dynamic"
        programming-language=${this.programmingLanguage}
      >
        <div slot="body">
          <stencila-ui-node-authors type="CodeInline">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="CodeInline"
            code=${this.code}
            code-authorship=${this.codeAuthorship}
            language=${this.programmingLanguage}
            read-only
          >
          </stencila-ui-node-code>
          <stencila-ui-node-outputs type="CodeInline">
            <slot name="outputs">${this.code}</slot>
          </stencila-ui-node-outputs>
        </div>
        <span slot="content"> ${this.code} </span>
      </stencila-ui-inline-on-demand>
    `
  }
}
