import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/instructions/instruction-messages'
import '../ui/nodes/properties/instructions/instruction-suggestions'

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

  @state()
  private showSuggestions = true

  /**
   * toggle the visibilty of the suggestions
   */
  private toggleSuggestions() {
    this.showSuggestions = !this.showSuggestions
  }

  override render() {
    return html`<stencila-ui-block-on-demand
      type=${this.type}
      node-id=${this.id}
      depth=${this.depth}
      ancestors=${this.ancestors}
    >
      <span slot="header-right" class="flex">
        <stencila-ui-node-execution-commands
          node-id=${this.id}
          type=${this.type}
        >
        </stencila-ui-node-execution-commands>
      </span>
      <div slot="body">
        <stencila-ui-node-execution-details
          type=${this.type}
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
      </div>
      <div slot="content" class="w-full">
        ${this.renderSuggestions()}
        <slot name="content"></slot>
      </div>
    </stencila-ui-block-on-demand>`
  }

  protected renderSuggestions() {
    const styles = apply([
      this.showSuggestions
        ? 'opacity-100 max-h-[10000px]'
        : 'opacity-0 max-h-0',
      'transition-all',
    ])

    return html`
      <div>
        ${this.context.cardOpen ? this.renderSuggestionsToggle() : ''}
        <div class=${styles}>
          <slot name="suggestions"></slot>
        </div>
      </div>
    `
  }

  protected renderSuggestionsToggle() {
    return html`
      <div>
        <button
          class="flex items-center gap-x-1 text-gray-300 hover:text-blue-500"
          @click=${() => this.toggleSuggestions()}
        >
          <sl-icon name=${this.showSuggestions ? 'eye' : 'eye-slash'}></sl-icon>
          <span class="text-sm">
            ${this.showSuggestions ? 'Hide' : 'Show'}${' '}suggestions
          </span>
        </button>
      </div>
    `
  }
}
