import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `InstructionMessage` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-message.md
 */
@customElement('stencila-instruction-message')
@withTwind()
export class InstructionMessage extends Entity {
  override render() {
    // TODO: Currently just showing `parts` property, not `role`, `authors` and `provenance`.
    return html`
      <div>
        <div class="py-2">
          <slot name="parts"></slot>
        </div>
        <div>
          <slot name="authors"></slot>
        </div>
      </div>
    `
  }
}
