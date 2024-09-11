import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Instruction } from './instruction'

import '../ui/nodes/cards/inline-on-demand'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'

/**
 * Web component representing a Stencila Schema `InstructionInline` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-inline.md
 */
@customElement('stencila-instruction-inline')
@withTwind()
export class InstructionInline extends Instruction {
  override render() {
    return html` <stencila-ui-inline-on-demand
      type="InstructionInline"
      node-id=${this.id}
    >
      <div slot="body">
        <stencila-ui-node-execution-details
          type="InstructionInline"
          mode=${this.executionMode}
          .tags=${this.executionTags}
          status=${this.executionStatus}
          required=${this.executionRequired}
          count=${this.executionCount}
          ended=${this.executionEnded}
          duration=${this.executionDuration}
        >
          <slot name="execution-dependencies"></slot>
          <slot name="execution-dependants"></slot>
        </stencila-ui-node-execution-details>

        <stencila-ui-node-execution-messages type="InstructionInline">
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <slot name="message"></slot>
      </div>
      <span slot="content">
        <slot name="suggestions"></slot>
      </span>
    </stencila-ui-inline-on-demand>`
  }
}
