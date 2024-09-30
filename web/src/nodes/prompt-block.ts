import { html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-in-flow'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/provenance'

import { Executable } from './executable'

/**
 * Web component representing a Stencila Schema `PromptBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/prompt-block.md
 */
@customElement('stencila-prompt-block')
@withTwind()
export class PromptBlock extends Executable {
  /**
   * Toggle show/hide content
   *
   * Defaults to true, and then is toggled off/on by user or
   * by changes to the prompt status.
   */
  @state()
  private showContent?: boolean = true

  override render() {
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
