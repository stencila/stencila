import { html } from 'lit'
import { property } from 'lit/decorators.js'

import { Executable } from './executable'

/**
 * Abstract base class for web components representing Stencila Schema `Instruction` node types
 * 
 * The only difference between the two node types that extend this, `InstructionBlock`
 * and `InstructionInline`, is the type of the `content` and `suggestion` slots.
 * Given that, even the `render()` method should be able to be shared between the two.  
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction.md
 */
export abstract class Instruction extends Executable {
  @property({ type: Array })
  candidates?: string[]

  @property()
  assignee?: string

  render() {
    return html`
      <slot name="content"></slot>

      <!-- TODO: implement design exposing '@property's and these <slot>s -->
      <div hidden>
        <slot name="authors" slot="authors"></slot>
        <slot name="messages" slot="messages"></slot>
        <slot name="suggestion" slot="suggestion"></slot>
      </div>
    `
  }
}
