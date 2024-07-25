import { SuggestionStatus } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

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
    return html`<stencila-ui-block-in-flow
      type="SuggestionBlock"
      node-id=${this.id}
      ?collapsed=${true}
    >
      <div slot="body">
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
