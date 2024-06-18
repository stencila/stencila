import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Instruction } from './instruction'

import '../ui/nodes/node-card/on-demand/in-line'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/provenance/provenance'

/**
 * Web component representing a Stencila Schema `InstructionInline` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-inline.md
 */
@customElement('stencila-instruction-inline')
@withTwind()
export class InstructionInline extends Instruction {
  override type: NodeType = 'InstructionInline'

  override render() {
    return html` <stencila-ui-inline-on-demand
      type="InstructionInline"
      view="dynamic"
      node-id=${this.id}
    >
      <div slot="body">
        <stencila-ui-node-execution-details
          type=${this.type}
          auto-exec=${this.autoExec}
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

        <stencila-ui-node-authors type="InstructionInline">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
        <stencila-ui-node-provenance type="InstructionInline">
          <slot name="provenance"></slot>
        </stencila-ui-node-provenance>
        <stencila-ui-node-execution-messages
          type=${this.type}
          warning-count=${this.warningCount}
          error-count=${this.errorCount}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <stencila-ui-node-instruction-messages type=${this.type}>
          <slot name="messages"></slot>
        </stencila-ui-node-instruction-messages>
      </div>
      <span slot="content">
        <slot name="suggestion"></slot>
      </span>
    </stencila-ui-inline-on-demand>`
  }
}
