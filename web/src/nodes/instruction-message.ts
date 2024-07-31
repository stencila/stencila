import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance/provenance'

/**
 * Web component representing a Stencila Schema `InstructionMessage` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-message.md
 */
@customElement('stencila-instruction-message')
@withTwind()
export class InstructionMessage extends Entity {
  override render() {
    return html`
      <div>
        <div class="py-2">
          <slot name="parts"></slot>
        </div>

        <stencila-ui-node-authors type="InstructionMessage">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
      </div>
    `
  }
}
