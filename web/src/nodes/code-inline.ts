import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../ui/nodes/node-card/on-demand/in-line'
import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/outputs'

import { withTwind } from '../twind'

import { CodeStatic } from './code-static'

@customElement('stencila-code-inline')
@withTwind()
export class CodeInline extends CodeStatic {
  /**
   * In dynamic view, also render the authors
   */
  override render() {
    return html`
      <stencila-ui-inline-on-demand
        disable-content-styles
        type="CodeInline"
        view="dynamic"
        programming-language=${this.programmingLanguage}
      >
        <div slot="body">
          <stencila-ui-node-authors type="CodeInline">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
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
        </div>
        <span slot="content">
          <slot></slot>
        </span>
      </stencila-ui-inline-on-demand>
    `
  }
}
