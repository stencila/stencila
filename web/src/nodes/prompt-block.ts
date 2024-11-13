import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Executable } from './executable'

import '../ui/nodes/cards/block-in-flow'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `PromptBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/prompt-block.md
 */
@customElement('stencila-prompt-block')
@withTwind()
export class PromptBlock extends Executable {
  @property()
  prompt: string

  /**
   * Toggle show/hide content
   *
   * Defaults to true, and then is toggled off/on by user.
   */
  @state()
  private showContent?: boolean = true

  override connectedCallback(): void {
    super.connectedCallback()
    this.showContent = !this.ancestors.endsWith('InstructionBlock')
  }

  override render() {
    if (this.ancestors.includes('StyledBlock')) {
      return html`
        <div class="w-full ${this.showContent ? '' : 'hidden'}">
          <slot name="content"></slot>
        </div>
      `
    }

    if (this.ancestors.endsWith('InstructionBlock')) {
      const { borderColour, colour } = nodeUi('InstructionBlock')

      return html`<div
          class="border-t border-[${borderColour}] bg-[${colour}] px-3 py-2 flex justify-between"
        >
          <span class="flex flex-row items-center gap-2">
            <stencila-ui-icon name="cardText"></stencila-ui-icon>
            <span class="font-sans text-xs">Prompt</span>
          </span>

          <span class="flex flex-row items-center">
            <span class="font-mono text-xs">${this.prompt}</span>
            <stencila-ui-chevron-button
              class="ml-4"
              default-pos=${this.showContent ? 'down' : 'left'}
              .clickEvent=${() => (this.showContent = !this.showContent)}
            ></stencila-ui-chevron-button>
          </span>
        </div>

        <div class="w-full bg-white/70 p-3 ${this.showContent ? '' : 'hidden'}">
          <slot name="content"></slot>
        </div>`
    }

    return html`<stencila-ui-block-in-flow
      type="PromptBlock"
      node-id=${this.id}
    >
      <span slot="header-right">
        <stencila-ui-node-execution-commands
          type="PromptBlock"
          node-id=${this.id}
        >
          <sl-tooltip
            content=${this.showContent ? 'Hide content' : 'Show content'}
          >
            <stencila-ui-icon-button
              class="ml-3"
              name=${this.showContent ? 'eyeSlash' : 'eye'}
              @click=${(e: Event) => {
                // Stop the click behavior of the card header parent element
                e.stopImmediatePropagation()
                this.showContent = !this.showContent
              }}
            ></stencila-ui-icon-button>
          </sl-tooltip>
        </stencila-ui-node-execution-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="PromptBlock"
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

        <stencila-ui-node-execution-messages type="PromptBlock">
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>
      </div>

      <div slot="content" class="w-full ${this.showContent ? '' : 'hidden'}">
        <slot name="content"></slot>
      </div>
    </stencila-ui-block-in-flow>`
  }
}
