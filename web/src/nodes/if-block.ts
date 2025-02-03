import { html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

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
  @state()
  hasClauses: boolean = true

  protected handleClauseChange(e: Event) {
    const slot = e.target as HTMLSlotElement
    this.hasClauses = slot.assignedElements().length > 0
  }

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`<slot name="clauses"></slot>`
    }

    // render with the `insert` chip in model chat response
    if (this.isWithinModelChatMessage()) {
      return html`
        <div class="group relative">
          ${this.renderInsertChip()} ${this.renderCard()}
        </div>
      `
    }

    return this.renderCard()
  }

  private renderCard() {
    return html`
      <stencila-ui-block-on-demand
        type="IfBlock"
        node-id=${this.id}
        depth=${this.depth}
        ?removeContentPadding=${true}
        ?noVisibleContent=${!this.hasClauses}
      >
        <div slot="header-right">
          <stencila-ui-node-execution-commands
            type="IfBlock"
            node-id=${this.id}
            depth=${this.depth}
            status=${this.executionStatus}
            required=${this.executionRequired}
          >
          </stencila-ui-node-execution-commands>
        </div>

        <div slot="body" class="h-full">
          <stencila-ui-node-execution-details
            type="IfBlock"
            node-id=${this.id}
            mode=${this.executionMode}
            bounds=${this.executionBounds}
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
