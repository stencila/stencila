import { html } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/provenance'

import { Executable } from './executable'

/**
 * Web component representing a Stencila Schema `IfBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-block.md
 */
@customElement('stencila-if-block')
@withTwind()
export class IfBlock extends Executable {
  @query('slot[name="clauses"]')
  clausesSlot!: HTMLSlotElement

  @state()
  hasClauses: boolean = true

  protected handleClauseChange() {
    this.hasClauses = this.clausesSlot.assignedElements().length > 0
  }

  override render() {
    if (this.ancestors.includes('StyledBlock') || this.isUserChatNode()) {
      return html`<slot name="clauses"></slot>`
    }

    return html`
      <stencila-ui-block-on-demand
        type="IfBlock"
        depth=${this.depth}
        ancestors=${this.ancestors}
        node-id=${this.id}
        ?removeContentPadding=${true}
        ?noVisibleContent=${!this.hasClauses}
      >
        <span slot="header-right">
          <stencila-ui-node-execution-commands
            type="IfBlock"
            node-id=${this.id}
          >
          </stencila-ui-node-execution-commands>
        </span>

        <div slot="body" class="h-full">
          <stencila-ui-node-execution-details
            type="IfBlock"
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

          <stencila-ui-node-authors type="IfBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-execution-messages type="IfBlock">
            <slot name="execution-messages"></slot>
          </stencila-ui-node-execution-messages>
        </div>

        <div slot="content">
          <slot name="clauses" @slotchange=${this.handleClauseChange}></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
