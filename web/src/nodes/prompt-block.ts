import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Executable } from './executable'

import '../ui/nodes/properties/generic/collapsible-details'
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
   * The node type of the parent node
   */
  private parentNodeType: NodeType

  /**
   * Toggle show/hide content
   *
   * Defaults to true, and then is toggled off/on by user.
   */
  private showContent?: boolean = true

  override connectedCallback(): void {
    super.connectedCallback()

    this.parentNodeType = this.ancestors.split('.').pop() as NodeType

    this.showContent = !(
      this.parentNodeType == 'Chat' || this.parentNodeType == 'InstructionBlock'
    )
  }

  override render() {
    if (
      this.ancestors.includes('StyledBlock') ||
      this.isUserChatMessageNode()
    ) {
      return html`
        <div class="w-full ${this.showContent ? '' : 'hidden'}">
          <slot name="content"></slot>
        </div>
      `
    }

    if (
      this.parentNodeType == 'Chat' ||
      this.parentNodeType == 'InstructionBlock'
    ) {
      return html`
        <stencila-ui-node-collapsible-details
          type=${this.parentNodeType}
          icon-name="compass"
          header-title="Prompt"
          ?expanded=${this.showContent}
        >
          <div class="mx-4 font-mono text-xs" slot="header-content">
            ${this.prompt}
          </div>
          <div class="w-full p-3" style="color: var(--default-text-colour);">
            <slot name="content"></slot>
          </div>
        </stencila-ui-node-collapsible-details>
      `
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
          node-id=${this.id}
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
