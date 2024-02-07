import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { property } from 'lit/decorators.js'

import { Executable } from './executable'
import './helpers/node-card'
import './helpers/node-authors'

/**
 * Abstract base class for web components representing Stencila Schema `Instruction` node types
 *
 * The only difference between the two node types that extend this, `InstructionBlock`
 * and `InstructionInline`, is the *type* of the `content` and `suggestion` slots.
 * Given that, even the `render()` method should be able to be shared between the two.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md
 */
export abstract class Instruction extends Executable {
  protected type: NodeType

  @property({ type: Array })
  candidates?: string[]

  @property()
  assignee?: string

  override render() {
    return html`
      ${this.documentView() !== 'source'
        ? html`<slot name="content"></slot>`
        : ''}

      <stencila-node-card type=${this.type}>
        <stencila-node-authors type=${this.type}>
          <slot name="authors"></slot>
        </stencila-node-authors>
        <div hidden>
          <!-- TODO -->
          <slot name="messages"></slot>
          <slot name="content"></slot>
          <slot name="suggestion"></slot>
        </div>
      </stencila-node-card>
    `
  }
}
