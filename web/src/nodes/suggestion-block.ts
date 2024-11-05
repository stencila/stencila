import { SuggestionStatus } from '@stencila/types'
import { css, html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

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
  suggestionStatus?: SuggestionStatus

  @property({ attribute: 'execution-ended', type: Number })
  executionEnded?: number

  @property({ attribute: 'execution-duration', type: Number })
  executionDuration?: number

  @property()
  feedback?: string

  @property()
  visible: boolean = false

  /**
   * Toggle show/hide content
   *
   * Defaults to true, and then is toggled off/on by user or
   * by changes to the suggestion status.
   */
  @state()
  private showContent?: boolean = true

  protected override update(changedProperties: PropertyValues): void {
    if (changedProperties.has('suggestionStatus')) {
      this.showContent = this.suggestionStatus === 'Accepted'
    }
    super.update(changedProperties)
  }

  static override styles = css`
    :host {
      flex: 0 0 100%;
      width: 100%;
    }
  `

  override render() {
    return html`<div
      class="${this.visible
        ? 'opacity-1 pointer-events-auto'
        : 'opacity-0 pointer-events-none'}"
    >
      <slot name="content"></slot>
    </div>`

    // Suggestion blocks are normally nested in an instruction block but can
    // be free standing
    const instructionId = this.closestGlobally('stencila-instruction-block')?.id

    return html`<stencila-ui-block-in-flow
      type="SuggestionBlock"
      node-id=${this.id}
      ?collapsed=${true}
    >
      <span slot="header-right">
        <stencila-ui-suggestion-commands
          type="SuggestionBlock"
          node-id=${this.id}
          instruction-id=${instructionId}
          suggestion-status=${this.suggestionStatus}
          feedback=${this.feedback}
        >
          <sl-tooltip
            content=${this.showContent ? 'Hide content' : 'Show content'}
          >
            <stencila-ui-icon-button
              name=${this.showContent ? 'eyeSlash' : 'eye'}
              @click=${(e: Event) => {
                // Stop the click behavior of the card header parent element
                e.stopImmediatePropagation()
                this.showContent = !this.showContent
              }}
            ></stencila-ui-icon-button>
          </sl-tooltip>
        </stencila-ui-suggestion-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="SuggestionBlock"
          ended=${this.executionEnded}
          duration=${this.executionDuration}
        >
        </stencila-ui-node-execution-details>

        <stencila-ui-node-authors type="SuggestionBlock" expanded>
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
      </div>

      <div slot="content" class="w-full ${this.showContent ? '' : 'hidden'}">
        <slot name="content"></slot>
      </div>
    </stencila-ui-block-in-flow>`
  }
}
