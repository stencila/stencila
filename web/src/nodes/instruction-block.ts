import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Instruction } from './instruction'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'

/**
 * Web component representing a Stencila Schema `InstructionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-block.md
 */
@customElement('stencila-instruction-block')
@withTwind()
export class InstructionBlock extends Instruction {
  override render() {
    const { borderColour } = nodeUi('InstructionBlock')

    return html`<stencila-ui-block-on-demand
      type="InstructionBlock"
      node-id=${this.id}
      depth=${this.depth}
      ancestors=${this.ancestors}
    >
      <span slot="header-right" class="flex">
        <stencila-ui-node-execution-commands
          node-id=${this.id}
          type="InstructionBlock"
        >
        </stencila-ui-node-execution-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="InstructionBlock"
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

        ${this.renderProperties()}

        <stencila-ui-node-execution-messages
          type="InstructionBlock"
          warning-count=${this.warningCount}
          error-count=${this.errorCount}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <slot name="model"></slot>

        <div class="border-t border-[${borderColour}]">
          <slot name="message"></slot>
        </div>
      </div>

      <div slot="content" class="w-full">
        <slot name="suggestions"></slot>
        <slot name="content"></slot>
      </div>
    </stencila-ui-block-on-demand>`
  }

  /**
   * Render a ribbon style container with properties of the instruction
   */
  private renderProperties() {
    const { borderColour, colour } = nodeUi('InstructionBlock')

    const styles = apply(
      'flex flex-row items-center',
      'px-3 py-1.5',
      `bg-[${colour}]`,
      'text-xs leading-tight font-sans',
      `border-t border-[${borderColour}]`
    )

    const inputStyles = apply([
      'text-sm text-black',
      'ml-2 px-2',
      'max-w-[33%]',
      'outline-black',
      `border border-[${borderColour}] rounded-sm`,
    ])

    return html`
      <div class=${styles}>
        <label>Assignee:</label>
        <input
          class="mr-3 ${inputStyles}"
          type="text"
          value=${this.assignee}
          ?readonly=${true}
        />
        <label>Replicates:</label>
        <input
          class=${inputStyles}
          type="number"
          value=${this.replicates}
          ?readonly=${true}
        />
      </div>
    `
  }
}
