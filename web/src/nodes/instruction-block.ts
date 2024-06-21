import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/instruction-messages'
import '../ui/nodes/properties/instruction-suggestions'

import { Instruction } from './instruction'

/**
 * Web component representing a Stencila Schema `InstructionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-block.md
 */
@customElement('stencila-instruction-block')
@withTwind()
export class InstructionBlock extends Instruction {
  override type: NodeType = 'InstructionBlock'

  override renderDynamicView() {
    return html`<stencila-ui-block-on-demand
      type=${this.type}
      view="dynamic"
      node-id=${this.id}
    >
      <span slot="header-right">
        <stencila-ui-node-execution-commands
          node-id=${this.id}
          type=${this.type}
        >
        </stencila-ui-node-execution-commands>
      </span>
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

        <stencila-ui-node-authors type="InstructionBlock">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

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

        <stencila-ui-node-instruction-suggestions type=${this.type}>
          <slot name="suggestions"></slot>
        </stencila-ui-node-instruction-suggestions>
      </div>
      <div slot="content" class="w-full">
        <slot name="content"></slot>
      </div>
    </stencila-ui-block-on-demand>`
  }
}
