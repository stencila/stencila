import { SuggestionStatus } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/commands/suggestion-commands'
import '../ui/nodes/cards/block-in-flow'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `SuggestionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/suggestion-block.md
 */
@customElement('stencila-suggestion-block')
@withTwind()
export class SuggestionBlock extends Entity {
  @property({ attribute: 'suggestion-status' })
  suggestionStatus: SuggestionStatus

  @property({ attribute: 'execution-ended', type: Number })
  executionEnded?: number

  @property({ attribute: 'execution-duration', type: Number })
  executionDuration?: number

  @property()
  feedback?: string

  override render() {
    const showSuggestion =
      !this.suggestionStatus || this.suggestionStatus === 'Proposed'

    const instructionId = this.closestGlobally('stencila-instruction-block').id

    return html`<stencila-ui-block-in-flow
      class=${!showSuggestion ? 'hidden' : ''}
      type="SuggestionBlock"
      node-id=${this.id}
      ?collapsed=${true}
    >
      <span slot="header-right">
        <stencila-ui-suggestion-commands
          type="SuggestionBlock"
          node-id=${this.id}
          instruction-id=${instructionId}
          feedback=${this.feedback}
        >
        </stencila-ui-suggestion-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="SuggestionBlock"
          ended=${this.executionEnded}
          duration=${this.executionDuration}
        >
        </stencila-ui-node-execution-details>

        <stencila-ui-node-authors type="SuggestionBlock">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
      </div>

      <div slot="content" class="w-full">
        <slot name="content"></slot>
      </div>
    </stencila-ui-block-in-flow>`
  }
}
