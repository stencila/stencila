import { provide } from '@lit/context'
import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { property, state } from 'lit/decorators.js'

import '../ui/nodes/card'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/instruction-messages'
import '../ui/nodes/properties/suggestion'
import { InstructionContext, instructionContext } from '../ui/nodes/context'

import { Executable } from './executable'

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

  @provide({ context: instructionContext })
  @state()
  protected context: InstructionContext = {
    nodeId: this.id,
    cardOpen: false,
  }

  override connectedCallback(): void {
    super.connectedCallback()
    this.shadowRoot.addEventListener(
      'collapsible-nodecard-toggle',
      (e: Event & { detail: InstructionContext }) => {
        this.context
        // only update the context for the relevant node
        if (e.detail.nodeId === this.id) {
          this.context = {
            ...this.context,
            cardOpen: e.detail.cardOpen,
          }
        }
      }
    )
  }

  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with execution actions and details and code read-only and collapsed.
   */
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

        <stencila-ui-node-authors type=${this.type}>
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
      </div>
      <div slot="content" class="w-full">
        <slot name="suggestion"></slot>
      </div>
    </stencila-ui-block-on-demand>`
  }

  /**
   * In source view render everything as in dynamic view except for
   * code, label, caption (because they are displayed in the source code).
   */
  override renderSourceView() {
    return html`<stencila-ui-node-card type=${this.type} view="source">
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

        <stencila-ui-node-authors type=${this.type}>
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
      </div>
    </stencila-ui-node-card>`
  }
}
