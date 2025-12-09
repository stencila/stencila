import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import '../ui/nodes/properties/code/code-static'

/**
 * Static version of the CodeBlock web component
 *
 * This is a lightweight alternative for static views that does not render node
 * cards and uses Prism.js instead of CodeMirror for syntax highlighting.
 */
@customElement('stencila-code-block')
export class CodeBlockStatic extends LitElement {
  @property()
  code: string

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string

  override render() {
    return html`
      <stencila-ui-node-code-static
        code=${this.code}
        language=${this.programmingLanguage}
      >
      </stencila-ui-node-code-static>
    `
  }
}
