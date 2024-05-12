import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { CodeStatic } from './code-static'

@customElement('stencila-code-block')
export class CodeBlock extends CodeStatic {
  /**
   * In static view, render the code and programming language
   *
   * TODO: display code as a read-only CodeMirror editor for syntax
   * highlighting and show language.
   */
  override renderStaticView() {
    return html`<slot name="code"></slot>`
  }

  /**
   * In dynamic view, also render the authors
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-block-on-demand
        type="CodeBlock"
        view="dynamic"
        programming-language=${this.programmingLanguage}
      >
        <div slot="body">
          <stencila-ui-node-authors type="CodeBlock">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="CodeBlock">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
        <div slot="content">
          <stencila-ui-node-code
            type="CodeBlock"
            code=${this.code}
            language=${this.programmingLanguage}
            read-only
          >
          </stencila-ui-node-code>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In dynamic view, render the authors but not code since that is
   * usually already rendered in the source.
   */
  override renderSourceView() {
    return html`
      <stencila-ui-block-in-flow type="CodeBlock" view="source">
        <div slot="body">
          <stencila-ui-node-authors>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="CodeBlock"
            code=${this.code}
            language=${this.programmingLanguage}
            read-only
          >
          </stencila-ui-node-code>
        </div>
      </stencila-ui-block-in-flow>
    `
  }
}
