import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `CodeExpression` node
 *
 * @slot output
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-expression.md
 */
@customElement('stencila-code-expression')
export class CodeExpression extends CodeExecutable {
  override render() {
    return html`<span>
      <slot name="output"></slot>
    </span>`
  }
}
