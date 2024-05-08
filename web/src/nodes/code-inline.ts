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
      <stencila-ui-inline-on-demand
        type="CodeInline"
        view="dynamic"
        title=${this.programmingLanguage}
      >
        <div slot="body">
          <stencila-ui-node-authors type="CodeInline">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="CodeInline"
            code=${this.code}
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

  /**
   * In dynamic view, render the authors but not code since that is
   * usually already rendered in the source.
   */
  override renderSourceView() {
    return html`
      <stencila-ui-block-in-flow
        type="CodeInline"
        view="source"
        title=${this.programmingLanguage}
      >
        <div slot="body">
          <stencila-ui-node-authors>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="CodeInline"
            code=${this.code}
            language=${this.programmingLanguage}
            read-only
          >
          </stencila-ui-node-code>
          <stencila-ui-node-outputs type="CodeInline">
            <slot name="outputs">${this.code}</slot>
          </stencila-ui-node-outputs>
        </div>
      </stencila-ui-block-in-flow>
    `
  }
}
