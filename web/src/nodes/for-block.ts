import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/programming-language'
import '../ui/nodes/properties/provenance'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `For` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for-block.md
 */
@customElement('stencila-for-block')
@withTwind()
export class ForBlock extends CodeExecutable {
  @property()
  variable: string

  @state()
  hasIterations: boolean = true

  private handleIterationChange(e: Event) {
    const slot = e.target as HTMLSlotElement
    this.hasIterations = !!slot.assignedElements()[0]
  }

  override render() {
    const { colour, borderColour } = nodeUi('ForBlock')

    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`<slot name="iterations"></slot>`
    }

    return html`
      <stencila-ui-block-on-demand
        type="ForBlock"
        node-id=${this.id}
        depth=${this.depth}
        ?removeContentPadding=${true}
        ?noVisibleContent=${!this.hasIterations}
      >
        <div slot="header-right">
          <stencila-ui-node-chat-commands
            type="ForBlock"
            node-id=${this.id}
            depth=${this.depth}
          >
          </stencila-ui-node-chat-commands>

          <stencila-ui-node-execution-commands
            type="ForBlock"
            node-id=${this.id}
            depth=${this.depth}
          >
          </stencila-ui-node-execution-commands>
        </div>

        <div slot="body" class="h-full">
          <stencila-ui-node-execution-details
            type="ForBlock"
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

          <stencila-ui-node-execution-messages type="ForBlock">
            <slot name="execution-messages"></slot>
          </stencila-ui-node-execution-messages>

          <stencila-ui-node-authors type="ForBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <div
            class="flex flex-row items-center gap-x-3 px-3 py-1.5 bg-[${colour}] border-t border-[${borderColour}]"
          >
            <span class="font-mono font-bold">for</span>

            <stencila-ui-node-code
              type="ForBlock"
              code=${this.variable}
              language=${this.programmingLanguage}
              execution-required=${this.executionRequired}
              read-only
              no-gutters
              container-classes="inline-block w-full border border-[${borderColour}] rounded overflow-hidden"
              class="flex-grow flex items-center max-w-[30%]"
            >
            </stencila-ui-node-code>

            <span class="font-mono font-bold">in</span>

            <stencila-ui-node-code
              type="ForBlock"
              code=${this.code}
              language=${this.programmingLanguage}
              read-only
              no-gutters
              container-classes="inline-block w-full border border-[${borderColour}] rounded overflow-hidden"
              class="flex-grow flex items-center max-w-[60%]"
            >
            </stencila-ui-node-code>

            ${this.programmingLanguage
              ? html`<stencila-ui-node-programming-language
                  programming-language=${this.programmingLanguage}
                ></stencila-ui-node-programming-language>`
              : ''}
          </div>
        </div>

        <div slot="content">
          <slot
            name="iterations"
            @slotchange=${this.handleIterationChange}
          ></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
